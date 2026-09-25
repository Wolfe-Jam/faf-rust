#!/usr/bin/env bash
# Build the npm faf-scoring-kernel package (Node target) from crates/faf-wasm-sdk.
# Usage: bash npm/faf-scoring-kernel/build.sh   → then `npm publish` from this folder (via /pubpro).
set -euo pipefail
export PATH="${HOME}/.cargo/bin:${PATH:-}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$(mktemp -d)"
wasm-pack build "$root/crates/faf-wasm-sdk" --release --target nodejs --out-dir "$out" --out-name faf_wasm_sdk
for f in faf_wasm_sdk.js faf_wasm_sdk.d.ts faf_wasm_sdk_bg.wasm faf_wasm_sdk_bg.wasm.d.ts; do cp "$out/$f" "$here/$f"; done
rm -rf "$out"
node -e "const k=require('$here/faf_wasm_sdk.js');const r=JSON.parse(k.score_faf('project:\n  name: t'));if(r.total!==33)throw new Error('score_faf total '+r.total+' != 33');console.log('built · sdk_version',k.sdk_version(),'· score_faf total',r.total)"
