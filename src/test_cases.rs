use lazy_static::lazy_static;
use crate::successor::*;

lazy_static! {
    pub static ref RANDOM_F64: Vec<f64> = {
        let mut v = vec![
            // Start with some fairly innocuous examples...
            0.1,
            -0.1,
            0.4,
            -0.4,
            0.0001,
            0.01,
            0.5,
            100.0,
            -100.0,
            0.2,
            0.3,
            -0.2,
            -0.3,

            // Pain
            f64::NAN,
            f64::INFINITY,
            -f64::INFINITY,
            0.0,
            -0.0,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE
        ];

        // Continued pain
        for exp in -1074..=1023 {
            let p = f64::powf(2.0, exp as f64);

            v.push(p);
            v.push(successor_f64(p));
            v.push(predecessor_f64(p));
        }

        v
    };
}
