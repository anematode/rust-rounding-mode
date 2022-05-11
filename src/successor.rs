// Getting the predecessor and successor of floats. Source: https://rust-lang.github.io/rfcs/3173-float-next-up-down.html

/// Returns the least number greater than `f`.
///
/// Let `TINY` be the smallest representable positive `f32`. Then,
///  - if `f.is_nan()`, this returns `f`;
///  - if `f` is `NEG_INFINITY`, this returns `-MAX`;
///  - if `f` is `-TINY`, this returns -0.0;
///  - if `f` is -0.0 or +0.0, this returns `TINY`;
///  - if `f` is `MAX` or `INFINITY`, this returns `INFINITY`;
///  - otherwise the unique least value greater than `f` is returned.
///
/// The identity `x.next_up() == -(-x).next_down()` holds for all `x`. When `x`
/// is finite `x == x.next_up().next_down()` also holds.
pub fn successor_f32(f: f32) -> f32 {
    const TINY_BITS: u32 = 0x1; // Smallest positive f32.
    const CLEAR_SIGN_MASK: u32 = 0x7fff_ffff;

    let bits = f.to_bits();
    if f.is_nan() || bits == f32::INFINITY.to_bits() {
        return f;
    }
    
    let abs = bits & CLEAR_SIGN_MASK;
    let next_bits = if abs == 0 {
        TINY_BITS
    } else if bits == abs {
        bits + 1
    } else {
        bits - 1
    };
    f32::from_bits(next_bits)
}

/// Returns the greatest number less than `f`.
///
/// Let `TINY` be the smallest representable positive `f64`. Then,
///  - if `f.is_nan()`, this returns `f`;
///  - if `f` is `INFINITY`, this returns `MAX`;
///  - if `f` is `TINY`, this returns 0.0;
///  - if `f` is -0.0 or +0.0, this returns `-TINY`;
///  - if `f` is `-MAX` or `NEG_INFINITY`, this returns `NEG_INFINITY`;
///  - otherwise the unique greatest value less than `f` is returned.
///
/// The identity `x.next_down() == -(-x).next_up()` holds for all `x`. When `x`
/// is finite `x == x.next_down().next_up()` also holds.
pub fn predecessor_f32(f: f32) -> f32 {
    const NEG_TINY_BITS: u32 = 0x8000_0001; // Smallest (in magnitude) negative f64.
    const CLEAR_SIGN_MASK: u32 = 0x7fff_ffff;

    let bits = f.to_bits();
    if f.is_nan() || bits == f32::NEG_INFINITY.to_bits() {
        return f;
    }
    
    let abs = bits & CLEAR_SIGN_MASK;
    let next_bits = if abs == 0 {
        NEG_TINY_BITS
    } else if bits == abs {
        bits - 1
    } else {
        bits + 1
    };
    f32::from_bits(next_bits)
}

/// Returns the least number greater than `f`.
///
/// Let `TINY` be the smallest representable positive `f64`. Then,
///  - if `f.is_nan()`, this returns `f`;
///  - if `f` is `NEG_INFINITY`, this returns `-MAX`;
///  - if `f` is `-TINY`, this returns -0.0;
///  - if `f` is -0.0 or +0.0, this returns `TINY`;
///  - if `f` is `MAX` or `INFINITY`, this returns `INFINITY`;
///  - otherwise the unique least value greater than `f` is returned.
///
/// The identity `x.next_up() == -(-x).next_down()` holds for all `x`. When `x`
/// is finite `x == x.next_up().next_down()` also holds.
pub fn successor_f64(f: f64) -> f64 {
    const TINY_BITS: u64 = 0x1; // Smallest positive f64.
    const CLEAR_SIGN_MASK: u64 = 0x7fff_ffff_ffff_ffff;

    let bits = f.to_bits();
    if f.is_nan() || bits == f64::INFINITY.to_bits() {
        return f;
    }
    
    let abs = bits & CLEAR_SIGN_MASK;
    let next_bits = if abs == 0 {
        TINY_BITS
    } else if bits == abs {
        bits + 1
    } else {
        bits - 1
    };
    f64::from_bits(next_bits)
}

/// Returns the greatest number less than `f`.
///
/// Let `TINY` be the smallest representable positive `f64`. Then,
///  - if `f.is_nan()`, this returns `f`;
///  - if `f` is `INFINITY`, this returns `MAX`;
///  - if `f` is `TINY`, this returns 0.0;
///  - if `f` is -0.0 or +0.0, this returns `-TINY`;
///  - if `f` is `-MAX` or `NEG_INFINITY`, this returns `NEG_INFINITY`;
///  - otherwise the unique greatest value less than `f` is returned.
///
/// The identity `x.next_down() == -(-x).next_up()` holds for all `x`. When `x`
/// is finite `x == x.next_down().next_up()` also holds.
pub fn predecessor_f64(f: f64) -> f64 {
    const NEG_TINY_BITS: u64 = 0x8000_0000_0000_0001; // Smallest (in magnitude) negative f64.
    const CLEAR_SIGN_MASK: u64 = 0x7fff_ffff_ffff_ffff;

    let bits = f.to_bits();
    if f.is_nan() || bits == f64::NEG_INFINITY.to_bits() {
        return f;
    }
    
    let abs = bits & CLEAR_SIGN_MASK;
    let next_bits = if abs == 0 {
        NEG_TINY_BITS
    } else if bits == abs {
        bits - 1
    } else {
        bits + 1
    };
    f64::from_bits(next_bits)
}

/// Returns whether two floating-point numbers have precisely the same bit pattern
pub fn identical_f64(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

/// Returns whether two floating-point numbers have precisely the same bit pattern
pub fn identical_f32(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

/// Mask out a float's various parts, for non-zero finite numbers only. Exponent is
/// unbiased. Sign is stored as a 64 bit integer. Mantissa has a leading one
/// placed if appropriate, and is guaranteed to be in the range [2^53, 2^54 - 1].
pub fn sign_exp_mant_f64(a: f64) -> (u64, i32, u64) {
    let bits = a.to_bits();
    let exp = ((bits & 0x7ff0_0000_0000_0000) >> 52) as i32 - 1023; // bias denier
    let sign = bits & (1u64 << 63);
    let mut mant = (bits & 0x000f_ffff_ffff_ffff);

    if exp >= -1022 {
    (
        sign,
        exp,
        mant + (1 << 52) // implicit upper bit
    )
    } else {
        let lz = mant.leading_zeros() as i32;
        let exp = -1022 - lz + 12; // bias denier

        mant <<= lz - 12;

        (
            sign,
            exp, 
            mant
        )
    }
}

/// This function assumes the mantissa has exactly 53 bits of precision (i.e., it is between 2^53
/// and 2^54 - 1), and that it is nonzero
pub fn from_sign_exp_mant_f64(sign: u64, mut exp: i32, mut mant: u64) -> f64 {
    if exp <= -1023 {
        // denormal enjoyer
        exp = -1023;
    }

    let exp = ((exp + 1023) as u64) << 52; // bias enjoyer
    let bits = sign + exp + mant;

    f64::from_bits(bits)
}

pub const MIN_SUBNORMAL_F64: f64 = 4.940656458412465442e-324;
