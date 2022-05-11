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

#[derive(PartialEq)]
enum NormalizeMode {
    TIES_EVEN,
    TIES_ODD,
    TIES_AWAY,
    TIES_ZERO,
    TRUNC,
    AWAY
}

/// We extract 53 bits into a u64 and round them accordingly, and return the appropriate shift
fn round_normalize_hilo(hi: u64, lo: u64, mode: &NormalizeMode) -> (u64, i32) {
    let mut new_mant: u64;
    let mut shift: i32;
    let mut trailing_behavior = 0i32; // 0 if zero, 1 if <1/2, 2 if tie, 3 if >1/2

    if hi != 64 {
        let hilz = hi.leading_zeros() as i32;
        shift = hilz - 64 - 11;

        let hi_shift = hilz - 11;

        if hi_shift < 0 {
            dbg!("Epic");
            new_mant = hi >> -hi_shift;
            let residue = hi - (new_mant << hi_shift);

            let tie = 1 << (-hi_shift - 1);

            if residue == 0 {

            } else if residue < tie {
                trailing_behavior = 1;
            } else if residue == tie {
                if lo == 0 {
                    trailing_behavior = 2;
                } else {
                    trailing_behavior = 3;
                }
            } else {
                trailing_behavior = 3;
            }

        } else if hi_shift == 0 {
            dbg!("Epi3c");
            new_mant = hi;
            
            if lo > 0 && lo < (1 << 63) {
                trailing_behavior = 1;
            } else if (lo == (1 << 63)) {
                trailing_behavior = 2;
            } else {
                trailing_behavior = 3;
            }
        } else {

            dbg!("Epic5");
            // Lower word is involved
            new_mant = (hi << hi_shift) + (lo >> -shift);
            let residue = lo - (lo >> -shift);
            let tie = 1 << (-shift - 1);

            if residue == 0 {
                
            } else if residue < tie {
                trailing_behavior = 1;
            } else if residue == tie {
                trailing_behavior = 2;
            } else {
                trailing_behavior = 3;
            }
        }
    } else {
        // Unlikely, where high word is 0
        
        let lolz = lo.leading_zeros() as i32;
        shift = lolz - 11;

        if lolz >= 11 {
            new_mant = lo << (lolz - 11);
        } else {
            new_mant = lo >> (11 - lolz);
            let trailing = lo - (new_mant << (11 - lolz));
            let tie = 1 << (10 - lolz);

            if trailing == 0 {
                
            } else if trailing < tie {
                trailing_behavior = 1;
            } else if trailing == tie {
                trailing_behavior = 2;
            } else {
                trailing_behavior = 3;
            }
        }
    }

    if mode == &NormalizeMode::TRUNC {
        return (new_mant, shift);
    }

    new_mant += if trailing_behavior > 0 {
        if mode == &NormalizeMode::AWAY {
            1
        } else {
            if trailing_behavior == 1 {
                0
            } else if trailing_behavior == 3 {
                1
            } else {
                // tie
                match mode {
                    NormalizeMode::TIES_AWAY => { 1 },
                    NormalizeMode::TIES_ZERO => { 0 },
                    other => {
                        if (new_mant & 1 == 0) == (&NormalizeMode::TIES_EVEN == mode) {
                            0
                        } else {
                            1
                        }
                    }
                }
            }
        }
    } else { 0 };

    (new_mant, shift)
}

// Assumes nonzero, finite
fn round_down_repr_to_f64(mut sign: u64, mut exp: i32, mut mul_hi: u64, mut mul_lo: u64) -> f64 {
    // Tiny or large exp
    if exp < -1074 {
        return if sign == 0 { 0. } else { -MIN_SUBNORMAL_F64 }
    }

    if exp > 1023 {
        return if sign == 0 { f64::MAX } else { -f64::INFINITY }
    }

    // Our procedure is straightforward: we count 53 bits in mul_hi/mul_lo, round, and shift
    // appropriately.
    let (truncated, shift) = round_normalize_hilo(mul_hi, mul_lo, if sign == 0 { &NormalizeMode::TRUNC } else { &NormalizeMode::AWAY });

    dbg!(truncated, shift);

    from_sign_exp_mant_f64(sign, exp + shift + 53, truncated)
}

/// Computes a rounded multiplication downward of two double-precision floating point numbers.
pub fn multiply_round_down(a: f64, b: f64) -> f64 {
    if !a.is_finite() || !b.is_finite() || a == 0. || b == 0. { // Rounding mode doesn't affect
        return a * b;
    }

    // The strategy is fairly straightforward: we multiply two 53-bit integers into a
    // full-precision 107-bit result
    let (sign, exp, mul_hi, mul_lo) = multiply_repr(a, b);

    round_down_repr_to_f64(sign, exp, mul_hi, mul_lo)
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
