
use rounding_mode::*;

fn main() {
    let a: f64 = 0.1;
    let b: f64 = -2.470328229206232721e-323;

    println!("Normal rounding: {:32e}", a * b);
    println!("Round down: {:32e}", native::mul_down(a, b));
    println!("Normal rounding: {:32e}", a * b);
}
