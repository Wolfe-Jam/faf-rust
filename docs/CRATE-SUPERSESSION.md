# FAF Rust crates — supersession & namespace map

**Owner:** [Wolfe-Jam](https://github.com/Wolfe-Jam) (FAF format steward)  
**Canonical publish home:** this workspace — [github.com/Wolfe-Jam/faf-rust](https://github.com/Wolfe-Jam/faf-rust)  
**As of:** 2026-08-01  

This document is **namespace stewardship**, not abandonment.  
FAF family names on [crates.io](https://crates.io) are held by the format owner so the ecosystem has one honest map. Older names stay reserved; **live install paths** are listed below.

---

## 1. Live foundation (edit & publish **only** here)

| crates.io | Version (live) | Role | Path in this repo |
|-----------|----------------|------|-------------------|
| [`faf-kernel`](https://crates.io/crates/faf-kernel) | 1.0.x | Parse, validate, Mk4 score | `crates/faf-kernel` |
| [`faf-fafb`](https://crates.io/crates/faf-fafb) | 1.0.x | FAFb binary brick | `crates/faf-fafb` |
| [`faf-rust-sdk`](https://crates.io/crates/faf-rust-sdk) | **3.x** | Thin facade → kernel + fafb | `crates/faf-rust-sdk` |
| [`faf-wasm-sdk`](https://crates.io/crates/faf-wasm-sdk) | **3.x** | WASM shell → kernel + fafb | `crates/faf-wasm-sdk` |

```toml
# Recommended application dependency
faf-rust-sdk = "3"
```

**Do not** publish foundation crates from the old standalone repos  
[`Wolfe-Jam/faf-rust-sdk`](https://github.com/Wolfe-Jam/faf-rust-sdk) or [`Wolfe-Jam/faf-wasm-sdk`](https://github.com/Wolfe-Jam/faf-wasm-sdk) — those trees are **historical and archived** on GitHub (2026-08-01); crates.io 3.x already points `repository` at **this** monorepo.

---

## 2. Live product crates (separate repos, same owner)

| crates.io | Role | Notes |
|-----------|------|--------|
| [`rust-faf-mcp`](https://crates.io/crates/rust-faf-mcp) | FAF MCP server (cargo registry type) | Product · BEST hop |
| [`mcp-better`](https://crates.io/crates/mcp-better) | BETTER textbook MCP (protocol 2026-07-28 / 7/28) | AAIF surface · not FAF DNA |
| [`faf`](https://crates.io/crates/faf) | Meta re-export convenience crate | Prefer pinning `faf-rust-sdk` 3.x directly when possible |
| [`faf-radio-rust`](https://crates.io/crates/faf-radio-rust) | Radio protocol client | Live secondary line |
| [`mcpaas`](https://crates.io/crates/mcpaas) | Related radio / MCPaaS client | Same family as radio |
| [`slash-tokens`](https://crates.io/crates/slash-tokens) | Token budget WASM utility | Side product |

---

## 3. Superseded → use instead

| crates.io name | Status | Use instead |
|----------------|--------|-------------|
| `faf-rust-sdk` **1.x / 2.x** | Prior majors (still downloadable) | **`faf-rust-sdk` 3.x** (this workspace) |
| Standalone git `faf-rust-sdk` @ 2.0.1 tree | **Archived** historical monorepo | **github.com/Wolfe-Jam/faf-rust** |
| Standalone git `faf-wasm-sdk` @ 2.x tree | **Archived** historical monorepo | **github.com/Wolfe-Jam/faf-rust** · crate **3.x** |
| `fafb` | Early name | **`faf-fafb`** |
| `faf-engine` | Early name | **`faf-kernel`** |
| `faf-mcp` | Early name | **`rust-faf-mcp`** |
| `faf-wasm` | Early name | **`faf-wasm-sdk`** |
| `faf-radio` | Early name | **`faf-radio-rust`** |
| `dotfaf` | Early tools name | **`faf-rust-sdk`** / **`faf-kernel`** |
| `faf-cli` (crates.io) | Name reserved; **not** the npm CLI | **npm `faf-cli`** · [faf.one](https://faf.one) |

---

## 4. Reserved family names (stewarded)

These crates.io names are **held by the FAF owner** so impostors and fork confusion do not own the brand. They are not for sale and not abandoned to strangers.

If a name has only an early `0.1.0` and no active product line, treat it as **reserved / superseded** per the table above until a future intentional release updates the description.

**Install the live row in §1–§2.** Do not depend on reserved stubs for production.

---

## 5. How to publish (maintainers)

1. Work only in **this** workspace for foundation crates.  
2. Order when needed: `faf-kernel` → `faf-fafb` → facades (`faf-rust-sdk`, `faf-wasm-sdk`).  
3. Product crates (`rust-faf-mcp`, `mcp-better`, …) publish from **their** repos.  
4. After any rename or major, update **this file** in the same PR.

---

## 6. Related

- Workspace README: [../README.md](../README.md)  
- Format home: [faf.one](https://faf.one)  
- IANA: `application/vnd.faf+yaml` (and FAF family media types)

*Stewardship map — own the names, tell the truth about which crate to install.*
