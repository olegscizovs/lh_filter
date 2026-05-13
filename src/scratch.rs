use nih_plug::prelude::*;

fn main() {
    let p = FloatParam::new("LP", 20000.0, FloatRange::Skewed { min: 20.0, max: 20000.0, factor: FloatRange::skew_factor(0.25) });
    println!("value: {}", p.value());
    println!("plain: {}", p.unmodulated_plain_value());
    println!("default: {}", p.default_plain_value());
}
