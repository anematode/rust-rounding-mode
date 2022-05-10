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
