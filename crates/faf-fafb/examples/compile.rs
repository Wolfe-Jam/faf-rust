//! `cargo run -p faf-fafb --example compile -- <file.faf> > out.fafb`
//! Deterministic (no timestamp) — used to (re)generate the golden fixture.
use std::io::Write;
fn main() {
    let path = std::env::args().nth(1).expect("usage: compile <file.faf>");
    let yaml = std::fs::read_to_string(&path).expect("read");
    let opts = faf_fafb::CompileOptions {
        use_timestamp: false,
    };
    let bytes = faf_fafb::compile(&yaml, &opts).expect("compile");
    std::io::stdout().write_all(&bytes).expect("write");
}
