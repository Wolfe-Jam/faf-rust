# CLAUDE.md - faf-rust-sdk

## Project
- **Name:** faf-rust-sdk
- **Version:** 1.2.0
- **Purpose:** Rust SDK for parsing, validating, and compressing FAF files
- **Registry:** crates.io (MIT)
- **Tests:** 137/137 passing (WJTTC 3-tier)

## Key Files
- src/lib.rs — core parser, validator, compressor
- src/binary/ — FAFb binary format support
- tests/ — WJTTC tier tests

## Commands
- cargo build
- cargo test
- cargo clippy -- -D warnings
