mod fallback;
mod modes;
mod native;

fn main() {
    println!("{}", 0.1 * 0.4);
    println!("{}", native::mul_down(0.1, 0.4));
}

