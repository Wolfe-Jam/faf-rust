<!-- agents:from-facts:start -->
<!-- authored by agents-md-facts — from your repo's facts, never guessed · re-run to refresh -->

# AGENTS.md — faf-rust

Rust

## Setup & build

```bash
cargo build --release    # build
```

## Run the tests

```bash
cargo test
cargo clippy
```

## Where things live

- `Cargo.toml`
- `README.md`

## Guardrails

- **Always OK:** read files, run the tests (`cargo test`), build the project.
- **Ask first:** dependency installs, deletions, migrations / schema changes.
- **Never:** force-push, push to `main`, commit secrets.

## Definition of Done

Done when: `cargo clippy` exits 0 · `cargo test` passes · committed with a clear message.

## Commit & PR

- Write a clear, descriptive commit message.
- Branch off `main`; never commit to `main` directly — open a PR for review.
- If build/test scripts or layout change, refresh this file in the **same PR** (`npx agents-md-facts`).
<!-- agents:from-facts:end -->
