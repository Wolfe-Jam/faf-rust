<!-- faf:start -->
<!-- faf: faf-rust-sdk | Rust | sdk | High-performance Rust SDK for FAF — parse, validate, score, .fafb binary. Zero-copy, inference-optimized. IANA-registered application/vnd.faf+yaml. A thin facade over faf-kernel + faf-fafb. -->
<!-- faf: claim=project.faf | family=FAF -->

# CLAUDE.md — faf-rust-sdk

## What This Is

High-performance Rust SDK for FAF — parse, validate, score, .fafb binary. Zero-copy, inference-optimized. IANA-registered application/vnd.faf+yaml. A thin facade over faf-kernel + faf-fafb.

## Stack

- **Language:** Rust
- **Runtime:** Native (Rust 2024 Edition)
- **Build:** Cargo (cargo build --release)
- **CI/CD:** GitHub Actions
- **Package Manager:** Cargo (crates.io)

## Context

- **Who:** Rust developers + FAF-family crates (xai-faf-rust, rust-faf-mcp) needing zero-copy .faf parse/validate/score
- **What:** Rust SDK for FAF — parser, validator, scorer, .fafb binary (string table, chunk classification, enterprise scale)
- **Why:** Zero-copy parsing optimized for inference workloads — the Rust-native FAF substrate
- **Where:** crates.io, GitHub (Wolfe-Jam/faf-rust-sdk)
- **When:** Production since November 2025 — now v3.1.0, IANA-registered (application/vnd.faf+yaml)
- **How:** cargo add faf-rust-sdk — parse/validate/score/compile via the crate API

---

*STATUS: BI-SYNC ACTIVE — 2026-08-26T22:54:20.897Z*
<!-- faf:end -->
