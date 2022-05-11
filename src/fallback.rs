use crate::successor::*;
use std::arch::asm;
use crate::native;

const MANTISSA_MASK: u64 = 0x000f_ffff_ffff_ffff;
const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;
const SIGN_MASK: u64 = 0x8000_0000_0000_0000;

/// Multiply two floats, returning the new exponent (as a bit pattern) and the exact mantissa as
/// a 128-bit integer, which will be chopped as appropriate. Assumed to be nonzero.
fn multiply_mantissas(a: f64, b: f64) -> (u64, u64) {
    let a = a.to_bits();
    let b = b.to_bits();

    let mut a_mant = a & MANTISSA_MASK;
    let mut b_mant = b & MANTISSA_MASK;

    // subnormal shenanigans
    if a & EXP_MASK != 0x0 {
        a_mant += 1 << 52;
    }

    if b & EXP_MASK != 0x0 {
        b_mant += 1 << 52;
    }

    dbg!(a_mant, b_mant);

    let mut hi: u64;
    let mut lo: u64;

    unsafe {
        // 64-bit unsigned multiplication into 128 bits. If there's an intrinsic for this I'd love
        // to know...
        asm!(
            "mul {}",
            in(reg) a_mant,
            inlateout("rax") b_mant => lo,
            lateout("rdx") hi,
            options(nomem, nostack, pure)
        );
    }

    (hi, lo)
}

fn round_down_quick(f: f64) -> f64 {
    // Using an and instruction allows less transferring between xmm and general registers
    if f.to_bits() & SIGN_MASK == 0 {
        // positive
        f64::from_bits(f.to_bits() - 1)
    } else {
        f64::from_bits(f.to_bits() + 1)
    }
}
 
/// Computes a rounded multiplication downward of two double-precision floating point numbers.
pub fn multiply_round_down(a: f64, b: f64) -> f64 {
    let original = a * b;
    if !a.is_finite() || !b.is_finite() || a == 0. || b == 0. { // Rounding mode doesn't affect
        return original;
    }

    if original == 0. {
        return if original.is_sign_positive() { original } else { -MIN_SUBNORMAL_F64 };
    } else if original.abs() == f64::INFINITY {
        return if original.is_sign_positive() { f64::MAX } else { original };
    }

    // Compute a full-precision result
    let (mul_hi, mul_lo) = multiply_mantissas(a, b);

    // It is possible for mul_hi to be zero (tiny subnormal multiplied with a large number), but
    // together there will always be at least 53 bits of stuff, because all pairs of subnormals were
    // dealt with at original == 0.0.

    // We now adjust the standard result based on what it "should" be based on the rounding
    
    if original.to_bits() & EXP_MASK != 0x0 {
        let original_m = original.to_bits() & MANTISSA_MASK;

        // Normal results, although it is conceivable that the answer rounded down is just barely
        // subnormal

        // By construction, since a, b < 2^53, mul_hi < 2^106 / 2^64 = 2^42, and therefore
        // lz >= 22, and the part of the mantissa that is rounded off lays entirely in
        // mul_lo.
        let mut lz = mul_hi.leading_zeros();

        // Unlikely, but possible
        if lz == 64 {
            lz += mul_lo.leading_zeros();
        }

        //   Hi                Lo
        // 0x000000843f ... 0x40291321
        //  <----->             ---->
        //   lz=23                truncated (lo_trunc_len=52)
        //        <----------->
        //          prec=53
        // By construction, lo_trunc_len is always nonnegative
        let lo_trunc_len = 128 - 53 - lz;
        let trunc_mask = (1u64 << lo_trunc_len) - 1;
        let trunc = mul_lo & trunc_mask;

        if trunc == 0 {
            // Original is likely correct, since the exact result only has 53 bits at all. There
            // is a tricky edge case, though: when original is the minimum positive normal, we
            // *could* be off by one. In this case, we check the entire sequence for being a power 
            // of two, in which case the original is correct; otherwise, we round down.
            return if original == f64::MIN_POSITIVE && (mul_hi.count_ones() + mul_lo.count_ones() != 1) {
                round_down_quick(original)
            } else { original };
        }

        // lo_trunc_len is now necessarily positive
        let tie = 1 << (lo_trunc_len - 1);

        // The truncated portion has three relevant possibilities: below tie, tie, and above
        // tie. Tie is the most tricky to deal with, but isn't impossible. If it is below a tie,
        // then we know the original "incorrectly" rounded upward in magnitude. If it is above
        // a tie, then we know the original "incorrectly" rounded downward in magnitude.

        if trunc != tie {
            if (original > 0.) == (trunc < tie) {
                // If original is positive and we rounded downward in mag, or if original is negative
                // and we rounded upward in mag, original is correct.
                return original;
            } else {
                // We incorrectly rounded upward in value. Because original is a normal number,
                // the issue can be rectified by either incrementing or decrementing the value
                // as an integer.
                return round_down_quick(original);
            }
        }

        // There was a tie; did the original round down (correct) or up (incorrect)? We examine the
        // bit before the end of 53-bit precision in the exact result. If it is even and the number
        // is positive, or if it is odd and the number is negative, then it rounded down, as 
        // desired. Otherwise, we need to round down ourselves
        let last_bit = mul_lo & (1u64 << lo_trunc_len);

        if (original > 0.) == (last_bit == 0) {
            return original;
        } else {
            return round_down_quick(original);
        }
    } else {
        // Subnormal results are trickier, since the place of rounding is hard to find. We defer
        // their processing to a separate function to deter inlining.
        multiply_round_down_subnormal(original, mul_hi, mul_lo)
    }
}

/// Handler for the case where the product is subnormal. Original is thus assumed to be subnormal
fn multiply_round_down_subnormal(original: f64, mul_hi: u64, mul_lo: u64) -> f64 {
    let original_m = original.to_bits() & MANTISSA_MASK;

    // Essentially, we calculate how many bits of accuracy the output has, then grab that many bits
    // from the exact calculation

    // The subnormal float has this many bits of precision 
    let end_prec = 64 - original_m.leading_zeros();

    // Beginning in the exact result
    let mut lz = mul_hi.leading_zeros();
    if lz == 64 {
        lz += mul_lo.leading_zeros();
    }

    let mut new_mant = 0u64;

    let mut end = lz + end_prec;
    if end < 64 {
        dbg!("Hi");
        // end of precision occurs within the first word
        let trunc_amt = 64 - end;

        new_mant = (mul_hi >> trunc_amt);
        let rem = mul_hi - (new_mant << trunc_amt);

        if rem == 0 {
            // exact
            return original;
        }

        if original < 0. {
            new_mant += 1;
        }
    } else if end == 64 {
        dbg!("Hi");
        new_mant = mul_hi;

        if mul_lo == 0 {
            // exact
            return original;
        }

        if original < 0. {
            new_mant += 1;
        }
    } else {

        dbg!("Fod");
        // occurs within the second word
        let lo_trunc_amt = 128 - end;
        new_mant = mul_lo >> lo_trunc_amt;
        let rem = mul_lo - (new_mant << lo_trunc_amt);
        new_mant += mul_hi << (end - 64); 

        if rem == 0 {
            // exact
            return original;
        }

        dbg!(rem, new_mant, mul_hi, mul_lo, lo_trunc_amt, end_prec);

        if original < 0. {
            new_mant += 1;
        }
    }

    if new_mant == 0x1 && (mul_hi.count_ones() + mul_lo.count_ones() != 1) {
        // Annoying edge case where the precision is "fake" in the sense that the rounded down
        // result is actually just 0. We detect this case with a popcount. The reason that the
        // popcount works is because of the range of relevant numbers
    
        new_mant = 0;
    }

    // With this scheme, a mantissa of 1 << 52 is implicitly taken to the maximum negative
    // normal number :P
    return f64::from_bits(new_mant).copysign(original);
}

#[cfg(test)]
mod tests {
    use crate::test_cases::*;
    use super::*;
    use crate::native;
    use crate::successor::*;

    /// Binary function between two f64s, e.g., multiplication rounding down
    type Binary64Fn<'a> = &'a dyn Fn(f64, f64) -> f64;

    /// Compare the behavior of two functions, throwing a bunch of random cases at them, ensuring
    /// they behave identically.
    fn compare_binary_f64_impl(expected: Binary64Fn, actual: Binary64Fn) {
        let mut cases = 0u64;
        for i in 0..RANDOM_F64.len() {
            for j in 0..RANDOM_F64.len() {
                let op1 = RANDOM_F64[i];
                let op2 = RANDOM_F64[j];

                let e = expected(op1, op2);
                let a = actual(op1, op2);

                assert!(identical_f64(e, a), "a = {:.18e}, b = {:.18e}, expected = {:.18e}, actual = {:.18e}", op1, op2, e, a);
                cases += 1;
            }
        }

        println!("Tested {} cases", cases);
    }

    #[test]
    fn test_multiply_round_down() {
        compare_binary_f64_impl(&native::mul_down, &multiply_round_down);
    }
}
