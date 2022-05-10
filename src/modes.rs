// Taken from https://help.totalview.io/previous_releases/2019/html/index.html#page/Reference_Guide/Intelx86MXSCRRegister.html

/// Enum for various rounding modes, most of which are supported by floating-point units.
pub enum Round {
    /// Ties to even, the default rounding mode in any program. Values are rounded to the nearest
    /// representable number; values falling precisely between two numbers are rounded to the value
    /// with a zero in the last place. This mode is supported by all floating-point units.
    TiesToEven = 0x0000,
    /// Rounds toward 0, i.e., positive values are rounded down to the nearest representable float,
    /// and negative values are rounded up to the nearest representable float. This mode is also
    /// known is truncation.
    TowardZero = 0x6000,
    /// Rounds upward, i.e., all values are rounded up to the nearest representable float.
    TowardPInf = 0x4000,
    /// Rounds downward, i.e., all values are rounded down to the nearest representable float.
    TowardNInf = 0x2000,
    /// Imposes no restrictions on the rounding; the value is within one ULP of the exact result.
    Faithful = -0x1,
    /// Ties away from 0, i.e., values midway between consecutive values are rounded up for
    /// positive numbers and below for negative numbers. This mode is not supported by any
    /// floating-point unit.
    TiesAway = -0x2,
    /// Ties to odd, analogous to ties to even, except that numbers midway between two
    /// representable values are rounded to the neighboring value with a one in the last place.
    /// This mode is not supported by any floating-point unit.
    TiesToOdd = -0x3
}

impl Round {
    /// Whether a rounding mode is natively supported. Additionally returns true for faithful
    /// rounding, which isn't a typical rounding mode per se, but imposes no restrictions on
    /// rounding.
    fn is_native(&self) -> bool {
        return *self < -0x1;
    }
}

#[cfg(test)]
mod tests {
    fn rounding_mode_is_native() {
        assert_eq!(Round::TiesToEven.is_native(), true);
        assert_eq!(Round::TowardZero.is_native(), true);
        assert_eq!(Round::TiesToOdd.is_native(), false);
        assert_eq!(Round::TiesToPInf.is_native(), true);
        assert_eq!(Round::TiesToNInf.is_native(), true);
        assert_eq!(Round::TiesAway.is_native(), false);
        assert_eq!(Round::TiesToOdd.is_native(), false);
    }
}
