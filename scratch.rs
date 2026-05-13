use nih_plug::prelude::*;

fn main() {
    let p = FloatParam::new("Q", 0.707, FloatRange::Linear { min: 0.1, max: 10.0 });
    println!("value: {}", p.value());
    println!("plain: {}", p.unmodulated_plain_value());
    println!("preview_plain(0.707): {}", p.preview_plain(0.707));
}
