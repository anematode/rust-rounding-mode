use std::arch::asm;
use crate::modes::*;

pub fn mul_down(mut a: f64, b: f64) -> f64 {
    unsafe {
        asm!(
            "sub rsp, 8",
            "stmxcsr [rsp]",
            "mov dword ptr [rsp + 4], 0x3F80", // Round down
            "ldmxcsr [rsp + 4]",
            "mulsd {a}, {b}",
            "ldmxcsr [rsp]",  // restore old state
            "add rsp, 8",
            a = inout(xmm_reg) a,
            b = in(xmm_reg) b
        );
    }

    return a;
}

/// Return the rounding mode that the MXCSR is currently in
pub fn get_rounding_mode() -> Round {
    let mut mxcsr: i32;

    unsafe {
        asm!(
            "sub rsp, 4",
            "ldmxcsr [rsp]",
            "mov {mxcsr}, [rsp]",
            "add rsp, 4",
            mxcsr = out(reg) mxcsr
        );

        let mode = mxcsr & 0xc00; // Brings into range of Round
        let mode: Round = ::std::mem::transmute(mode); 

        return mode;
    }
}

/// Panics if we are not in round-to-nearest mode again
fn ensure_state_restored() {
    match get_rounding_mode() {
        Round::TiesToEven => {

        },
        _ => {
            panic!("Failed to restore round-to-nearest rounding");
        }
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use super::*;

    #[derive(Clone)]
    struct BinaryTestCase {
        op1: f64,
        op2: f64,
        res: f64,
    }

    lazy_static! {
        static ref mul_rd_tests: Vec<BinaryTestCase> = vec![
            BinaryTestCase {
                op1: 0.1, op2: 0.4, res: 0.04
            },
            BinaryTestCase {
                op1: -0.1, op2: 0.4, res: -0.04000000000000001 
            }
        ];
    }

    fn same_float(a: f64, b: f64) -> bool {
        a.to_bits() == b.to_bits()
    }

    fn test_binary(cases: Vec<BinaryTestCase>, f: &dyn Fn(f64, f64) -> f64) {
        for case in cases {
            let res = f(case.op1, case.op2);

            assert!(same_float(res, case.res), "a = {}, b = {}, expected = {}, actual = {}", case.op1, case.op2, case.res, res);
        }

        ensure_state_restored()
    }

    #[test]
    fn test_mul_rd() {
        test_binary(mul_rd_tests.to_vec(), &mul_down)
    }
}
