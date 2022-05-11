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
        // Perform a 64-bit unsigned multiplication into 128 bits
        asm!(
            "mul {}",
            in(reg) a_mant,
            inlateout("rax") b_mant => lo,
            lateout("rdx") hi
        );
    }

    (sign, exp, hi, lo)
}

/// Computes a rounded multiplication downward of two double-precision floating point numbers.
pub fn multiply_round_down(a: f64, b: f64) -> f64 {
    if !a.is_finite() || !b.is_finite() || a == 0. || b == 0. { // Rounding mode doesn't affect
        return a * b;
    }

    // The strategy is fairly straightforward: we multiply two 53-bit integers into a
    // full-precision 106-bit result, and profit
    let (mut sign, mut exp, mut mul_hi, mut mul_lo) = multiply_repr(a, b);
    let mut mul_lo_cutoff = 53; // bit in mul_lo to be cut off

    // We check for overflow at the 107th bit, aka the 43rd bit of mul_hi
    let overflow = mul_hi & (1 << 43);
    if overflow != 0 {
        mul_lo_cutoff += 1;
        exp += 1;
    }

    let mut new_mant: u64;

    if sign == 0 {
        // Positive result, truncation will be done automatically
    } else {
        // Negative result, round up in magnitude if needed
        let mask = ((1u64 << mul_lo_cutoff) - 1);
        if mul_lo & mask != 0 {
            // Rounding necessary, result is inexact
            mul_lo &= !mask; // truncate
            mul_lo = mul_lo.wrapping_add(1u64 << mul_lo_cutoff); // round up

            if mul_lo == 0 { // overflow occurred
                mul_hi += 1;
            }
        }
    }
    
    // Compute new mantissa
    new_mant = (mul_lo >> mul_lo_cutoff) + (mul_hi << (64 - mul_lo_cutoff));
    
    // Tiny or large exp
    if exp <= -1074 {
        return if sign == 0 {
            // Rounds to 0
            0.
        } else {
            -f64::MIN_POSITIVE
        }
    }

    if exp > 1023 {
        return if sign == 0 {
            f64::MAX
        } else {
            -f64::INFINITY
        }
    } 

    // Since we're rounding down, we check if sign is positive or negative. If sign is 0 (positive)
    // then we truncate; if sign is 1 (negative) we round up if the second word
    from_sign_exp_mant_f64(sign, exp, new_mant)
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
                
                assert!(identical_f64(e, a), "a = {:.16e}, b = {:.16e}, expected = {:.16e}, actual = {:.16e}", op1, op2, e, a);
            }
        }
    }

    #[test]
    fn test_multiply_round_down() {
        compare_binary_f64_impl(&native::mul_down, &multiply_round_down);
    }
}
