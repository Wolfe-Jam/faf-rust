//! Quick scoring harness: `cargo run -p faf-kernel --example score -- <file.faf>`
use std::fs;
fn main() {
    let path = std::env::args().nth(1).expect("usage: score <file>");
    let yaml = fs::read_to_string(&path).expect("read");
    match faf_kernel::score(&yaml) {
        Ok(r) => println!(
            "faf-kernel: score={}% tier={} populated={} ignored={} active={} total={}",
            r.score, r.tier, r.populated, r.ignored, r.active, r.total
        ),
        Err(e) => println!("error: {}", e),
    }
}
