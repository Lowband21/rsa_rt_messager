mod key_gen;
use key_gen::gen_keys;
use std::time::Instant;
fn main() {
    let now = Instant::now();
    for _ in 0..1000 {
        gen_keys();
    }
    let millis = now.elapsed().as_millis();
    println!("Average elapsed time: {}", millis / 1000);
}
