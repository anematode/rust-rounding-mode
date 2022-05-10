mod fallback;

fn main() {
    println!("Hello, world!");
    println!("{}", fallback::multiply_round_up(0.0, 2.0));
}

