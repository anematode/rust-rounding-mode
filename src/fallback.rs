use crate::successor::*;
use std::arch::asm;

/// Multiply two floats, returning the new exponent (as a bit pattern) and the exact mantissa as
/// a 128-bit integer, which will be chopped as appropriate. Should only be provided with non-zero,
/// finite numbers.
fn multiply_repr(a: f64, b: f64) -> (u64, i32, u64, u64) {
    let (a_sign, a_exp, a_mant) = sign_exp_mant_f64(a);
    let (b_sign, b_exp, b_mant) = sign_exp_mant_f64(b);

    let sign = a_sign ^ b_sign;
    let exp = a_exp + b_exp; 

    let mut hi: u64;
    let mut lo: u64;

    unsafe {
        // 64-bit unsigned multiplication into 128 bits
        asm!(
            "mul {}",
            in(reg) a_mant,
            inlateout("rax") b_mant => lo,
            lateout("rdx") hi
        );
    }

    dbg!(a_exp, b_exp, sign, exp, hi, lo);
    (sign, exp, hi, lo)
}

const fn generate_mask(cnt: i32) -> u64 {
    if cnt == 0 {
        0x0
    } else {
        0xffff_ffff_ffff_ffff >> (64 - cnt)
    }
}

// Assumes nonzero, finite
fn round_exact_repr_to_f64(mut sign: u64 ,mut exp: i32, mut mul_hi: u64, mut mul_lo: u64) -> f64 {
    // Tiny or large exp
    if exp < -1074 {
        return if sign == 0 {
            // Rounds to 0
            0.
        } else {
            -MIN_SUBNORMAL_F64
        }
    }

    if exp > 1023 {
        return if sign == 0 {
            f64::MAX
        } else {
            -f64::INFINITY
        }
    }
    let mut mul_lo_cutoff = 52; // bit in mul_lo including and after which should be cut off

    // We check for overflow at the 107th bit, aka the 43rd bit of mul_hi
    let overflow = mul_hi & (1 << 41);
    if overflow != 0 {
        mul_lo_cutoff += 1;
    } else {
        exp -= 1;
    }

    let mut new_mant: u64;

    // Denormal pain
    if exp <= -1023 - 12 {
        let mul_hi_cutoff = -1023 - 12 - exp + (overflow != 0) as i32; // how much to chop off the HIGH part (the entire low part is discarded)
        println!("Shit {}", mul_hi_cutoff);

        new_mant = mul_hi >> mul_hi_cutoff; // truncate
        if sign != 0 {
            let mask = generate_mask(mul_hi_cutoff);

            if mul_hi & mask != 0 || mul_lo != 0 {
                println!("Epic");
                // Round up
                new_mant += 1;

                if new_mant >= (1 << (52 - mul_hi_cutoff)) { // Annoying overflow
                    println!("Fuck");
                    exp += 1;
                    new_mant >>= 1;
                }
            }
        }

        dbg!(new_mant, exp);
    } else {
        // Truncation still occurs in the second 64-bit part :)
        if (exp <= -1023) {
            mul_lo_cutoff += -1023 - exp;
        }

        dbg!(mul_lo_cutoff);

        if sign == 0 {
            // Positive result, truncation will be done automatically
        } else {
            // Negative result, round up in magnitude if needed
            let mask = generate_mask(mul_lo_cutoff); 
            if mul_lo & mask != 0 {
                // Rounding necessary, result is inexact
                mul_lo &= !mask; // truncate
                mul_lo = mul_lo.wrapping_add(1u64.checked_shl(mul_lo_cutoff as u32).unwrap_or(0)); // round up

                if mul_lo == 0 { // overflow occurred
                    mul_hi += 1;
                }
            }
        }

        new_mant = (mul_hi << (64 - mul_lo_cutoff)) + mul_lo.checked_shr(mul_lo_cutoff as u32).unwrap_or(0u64);
    }

    dbg!(new_mant);

    // Compute new mantissa    

    // Since we're rounding down, we check if sign is positive or negative. If sign is 0 (positive)
    // then we truncate; if sign is 1 (negative) we round up if the second word
    from_sign_exp_mant_f64(sign, exp, new_mant)

}

/// Computes a rounded multiplication downward of two double-precision floating point numbers.
pub fn multiply_round_down(a: f64, b: f64) -> f64 {
    if !a.is_finite() || !b.is_finite() || a == 0. || b == 0. { // Rounding mode doesn't affect
        return a * b;
    }

    // The strategy is fairly straightforward: we multiply two 53-bit integers into a
    // full-precision 107-bit result
    let (sign, exp, mul_hi, mul_lo) = multiply_repr(a, b);

    round_exact_repr_to_f64(sign, exp, mul_hi, mul_lo)
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
        for i in 0..RANDOM_F64.len() {
            for j in 0..RANDOM_F64.len() {
                let op1 = RANDOM_F64[i];
                let op2 = RANDOM_F64[j];

                let e = expected(op1, op2);
                let a = actual(op1, op2);

                assert!(identical_f64(e, a), "a = {:.18e}, b = {:.18e}, expected = {:.18e}, actual = {:.18e}", op1, op2, e, a);
            }
        }
    }

    #[test]
    fn test_multiply_round_down() {
        compare_binary_f64_impl(&native::mul_down, &multiply_round_down);
    }
}
