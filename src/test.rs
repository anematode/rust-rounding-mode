#[cfg(test)]
use super::*;

#[test]
fn rounding_mode_is_native() {
    assert_eq!(Round::TiesToEven.is_native(), true);
    assert_eq!(Round::TowardZero.is_native(), true);
    assert_eq!(Round::TiesToOdd.is_native(), false);
    assert_eq!(Round::TiesToPInf.is_native(), true);
    assert_eq!(Round::TiesToNInf.is_native(), true);
    assert_eq!(Round::TiesAway.is_native(), false);
    assert_eq!(Round::TiesToOdd.is_native(), false);
}
