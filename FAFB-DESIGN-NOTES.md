# .fafb Design Notes & Future Directions

> **Canonical spec:** [xai-faf-rust/BINARY-FORMAT.md](https://github.com/Wolfe-Jam/xai-faf-rust/blob/main/BINARY-FORMAT.md)
> **IANA MIME type:** `application/vnd.fafb`

This document contains early design ideas and future directions from the original
November 2025 research phase. The canonical FAFb v1.0 specification lives in
xai-faf-rust. Ideas below (embeddings layer, attention weights, consumption
patterns) represent potential v2 features, not the current implemented format.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  FAF BINARY FORMAT • DESIGN NOTES & FUTURE DIRECTIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Draft v0.1 • November 2025 • Uncharted Waters
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> "We're not breaking FAF. We're building what FAF sits on."

---

## Overview

The `.fafb` binary format is the compiled, AI-optimized representation of `.faf` files. It sits below the human-readable YAML layer, providing:

- **Instant access** - O(1) section lookup
- **Smart truncation** - Priority-based context window management
- **Pre-computed tokens** - No runtime estimation
- **Memory mapping** - Zero-copy loading

```
Human writes .faf (YAML)
          ↓
Rust SDK compiles → .fafb (binary)
          ↓
AI loads .fafb (optimized)
```

---

## File Extension

- **Primary**: `.fafb` (FAF Binary)
- **MIME type**: `application/vnd.fafb` (future IANA registration)

---

## Header Structure (32 bytes)

```rust
struct FafbHeader {
    // Identification (8 bytes)
    magic: [u8; 4],        // b"FAFB" - File type identifier
    version_major: u8,     // Format version (breaking changes)
    version_minor: u8,     // Format version (additions)
    flags: u16,            // Feature flags

    // Integrity (12 bytes)
    source_checksum: u32,  // CRC32 of original .faf YAML
    created_timestamp: u64, // Unix timestamp

    // Index (8 bytes)
    section_count: u16,    // Number of sections
    section_table_offset: u32, // Byte offset to section table
    reserved: u16,         // Future use

    // Size (4 bytes)
    total_size: u32,       // Total file size in bytes
}
```

### Magic Number

```
Bytes 0-3: 0x46 0x41 0x46 0x42 ("FAFB")
```

Any file not starting with these bytes is not a valid .fafb file.

---

## Feature Flags

```rust
// Bit flags for optional features (16 bits)
const FLAG_COMPRESSED: u16     = 0b0000_0000_0000_0001;  // Content is zstd compressed
const FLAG_EMBEDDINGS: u16     = 0b0000_0000_0000_0010;  // Contains pre-computed embeddings
const FLAG_TOKENIZED: u16      = 0b0000_0000_0000_0100;  // Contains token boundaries
const FLAG_WEIGHTED: u16       = 0b0000_0000_0000_1000;  // Contains attention weights
const FLAG_MODEL_HINTS: u16    = 0b0000_0000_0001_0000;  // Contains model-specific hints
const FLAG_SIGNED: u16         = 0b0000_0000_0010_0000;  // Contains cryptographic signature

// Reserved: bits 6-15 for future use
```

Readers MUST ignore unknown flags and continue processing.

---

## Section Table

Located at `section_table_offset`, contains `section_count` entries.

### Section Entry (16 bytes)

```rust
struct SectionEntry {
    section_type: u8,      // Section identifier
    priority: u8,          // 0-255, truncation priority
    offset: u32,           // Byte offset to section data
    length: u32,           // Section data length in bytes
    token_count: u16,      // Pre-computed token estimate
    flags: u16,            // Section-specific flags
}
```

### Section Types

```rust
// Core sections (0x01-0x0F)
const SECTION_META: u8        = 0x01;  // faf_version, name, score
const SECTION_TECH_STACK: u8  = 0x02;  // Languages, frameworks
const SECTION_KEY_FILES: u8   = 0x03;  // File list with descriptions
const SECTION_ARCHITECTURE: u8 = 0x04; // System design
const SECTION_COMMANDS: u8    = 0x05;  // Build/test/run commands
const SECTION_CONTEXT: u8     = 0x06;  // Additional context
const SECTION_BISYNC: u8      = 0x07;  // Bi-sync metadata

// Extended sections (0x10-0xFE)
const SECTION_EMBEDDINGS: u8  = 0x10;  // Pre-computed vectors
const SECTION_TOKEN_MAP: u8   = 0x11;  // Token boundary markers
const SECTION_MODEL_HINTS: u8 = 0x12;  // Model-specific optimization

// Custom (0xFF)
const SECTION_CUSTOM: u8      = 0xFF;  // User-defined sections
```

Readers MUST skip unknown section types gracefully.

---

## Priority System

Priority determines truncation order when context window is constrained.

```rust
// Priority levels (higher = more important)
const PRIORITY_CRITICAL: u8   = 255;  // Never truncate (name, version)
const PRIORITY_HIGH: u8       = 200;  // Truncate last (key_files, tech_stack)
const PRIORITY_MEDIUM: u8     = 128;  // Normal (architecture, commands)
const PRIORITY_LOW: u8        = 64;   // Truncate first (verbose context)
const PRIORITY_OPTIONAL: u8   = 0;    // Can be omitted entirely
```

### Default Priorities

| Section | Default Priority | Rationale |
|---------|------------------|-----------|
| META | 255 (Critical) | Identity - always needed |
| TECH_STACK | 200 (High) | Core context |
| KEY_FILES | 200 (High) | Navigation |
| COMMANDS | 180 (High) | Actionable |
| ARCHITECTURE | 128 (Medium) | Design context |
| CONTEXT | 64 (Low) | Supplementary |
| BISYNC | 32 (Low) | Metadata |

---

## Binary Layout

```
┌─────────────────────────────────┐  Offset 0
│  Header (32 bytes)              │
├─────────────────────────────────┤  Offset 32
│  Section 0 Data                 │
│  (variable length)              │
├─────────────────────────────────┤
│  Section 1 Data                 │
│  (variable length)              │
├─────────────────────────────────┤
│  ...                            │
├─────────────────────────────────┤
│  Section N Data                 │
│  (variable length)              │
├─────────────────────────────────┤  section_table_offset
│  Section Table                  │
│  (16 bytes × section_count)     │
├─────────────────────────────────┤
│  Optional: Embeddings           │  (if FLAG_EMBEDDINGS)
├─────────────────────────────────┤
│  Optional: Token Map            │  (if FLAG_TOKENIZED)
└─────────────────────────────────┘  total_size
```

### Design Rationale

Section table at END allows:
- Streaming writes (sections first, table last)
- Single-pass compilation
- Forward references resolved at end

---

## Section Data Encoding

Each section's data is UTF-8 encoded text (or binary for embeddings).

### META Section (0x01)

```
name_length: u16
name: [u8; name_length]
faf_version_length: u8
faf_version: [u8; faf_version_length]
score: u8  // 0-100
```

### TECH_STACK Section (0x02)

```
entry_count: u16
entries: [
  key_length: u8
  key: [u8; key_length]
  value_length: u16
  value: [u8; value_length]
] × entry_count
```

### KEY_FILES Section (0x03)

```
file_count: u16
files: [
  path_length: u16
  path: [u8; path_length]
  desc_length: u16
  description: [u8; desc_length]
] × file_count
```

---

## Token Estimation

Token count is estimated at compile time for context window budgeting.

```rust
fn estimate_tokens(data: &[u8]) -> u16 {
    // Rough estimate: 4 bytes per token (English text)
    // Capped at u16::MAX (65535)
    std::cmp::min(data.len() / 4, 65535) as u16
}
```

Model-specific token counts can be included in MODEL_HINTS section.

---

## Loading Strategies

### Full Load

```rust
let faf = FafBinary::load(data)?;
```

### Budget-Constrained Load

```rust
// Load highest-priority sections up to token budget
let faf = FafBinary::load_with_budget(data, 4096)?;
```

### Selective Load

```rust
// Load only specific sections
let faf = FafBinary::load_sections(data, &[
    SECTION_META,
    SECTION_KEY_FILES,
])?;
```

---

## Compilation Process

```
┌─────────────────┐
│  .faf (YAML)    │  Source of truth
└────────┬────────┘
         │
    ┌────▼────┐
    │  Parse  │  YAML → Faf struct
    └────┬────┘
         │
    ┌────▼────┐
    │  Score  │  Calculate priorities
    └────┬────┘
         │
    ┌────▼────┐
    │ Encode  │  Sections → bytes
    └────┬────┘
         │
    ┌────▼────┐
    │  Index  │  Build section table
    └────┬────┘
         │
    ┌────▼────┐
    │ Header  │  Write final header
    └────┬────┘
         │
┌────────▼────────┐
│  .fafb (binary) │  Optimized output
└─────────────────┘
```

---

## Rust Implementation

### Dependencies

```toml
[dependencies]
byteorder = "1.5"
crc32fast = "1.3"
```

### Core Structures

```rust
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

pub struct FafBinary {
    header: FafbHeader,
    sections: Vec<(SectionEntry, Vec<u8>)>,
}

impl FafBinary {
    pub const MAGIC: &'static [u8] = b"FAFB";

    /// Compile .faf to .fafb
    pub fn compile(faf: &Faf) -> Result<Vec<u8>, FafError> {
        // Implementation here
    }

    /// Load .fafb to Faf
    pub fn load(data: &[u8]) -> Result<Faf, FafError> {
        // Implementation here
    }

    /// Load with token budget
    pub fn load_with_budget(data: &[u8], budget: u16) -> Result<Faf, FafError> {
        // Implementation here
    }
}
```

---

## Validation Requirements

### Round-Trip Test

```rust
#[test]
fn test_roundtrip() {
    let original = Faf::parse(YAML_CONTENT)?;
    let binary = FafBinary::compile(&original)?;
    let recovered = FafBinary::load(&binary)?;

    assert_eq!(original, recovered);
}
```

### Corruption Detection

```rust
#[test]
fn test_invalid_magic() {
    let mut data = valid_fafb_data();
    data[0] = 0x00;  // Corrupt magic

    assert!(FafBinary::load(&data).is_err());
}

#[test]
fn test_checksum_mismatch() {
    let mut data = valid_fafb_data();
    data[8] ^= 0xFF;  // Corrupt checksum

    assert!(FafBinary::load(&data).is_err());
}
```

### Priority Truncation

```rust
#[test]
fn test_budget_truncation() {
    let faf = FafBinary::load_with_budget(&data, 100)?;

    // META (critical) should always be present
    assert!(faf.name.is_some());

    // Low-priority sections may be absent
    // (depends on their token counts)
}
```

---

## Versioning Strategy

### Format Version

- **Major** (breaking): Header structure, section table format
- **Minor** (additive): New section types, new flags

### Compatibility Rules

1. Readers MUST reject major version mismatch
2. Readers MUST accept unknown minor versions
3. Readers MUST skip unknown section types
4. Readers MUST ignore unknown flags

```rust
fn check_version(header: &FafbHeader) -> Result<(), FafError> {
    if header.version_major != CURRENT_MAJOR {
        return Err(FafError::IncompatibleVersion);
    }
    // Minor version mismatches are OK
    Ok(())
}
```

---

## Security Considerations

### Input Validation

- Validate all offsets are within file bounds
- Validate section lengths don't overflow
- Validate string lengths before allocation
- Cap maximum file size (suggested: 10MB)

### Denial of Service

- Limit section count (suggested: 256)
- Limit individual section size (suggested: 1MB)
- Validate token counts are reasonable

---

## Future Extensions

### Embeddings (FLAG_EMBEDDINGS)

Pre-computed vector representations for semantic search.

```rust
struct EmbeddingSection {
    model_id: [u8; 32],    // Model identifier
    dimensions: u16,       // Vector dimensions
    count: u32,            // Number of embeddings
    vectors: [f32; dimensions * count],
}
```

### Model Hints (FLAG_MODEL_HINTS)

Optimization hints for specific models.

```rust
struct ModelHint {
    model_pattern: String,  // e.g., "gpt-4*", "claude-*"
    token_count: u32,       // Exact token count for this model
    attention_weights: Vec<f32>,  // Per-section attention hints
}
```

### Compression (FLAG_COMPRESSED)

Section data compressed with zstd.

```rust
if header.flags & FLAG_COMPRESSED != 0 {
    data = zstd::decode_all(data)?;
}
```

---

## Breakage Analysis

### Safe Changes (Minor Version Bump)

- New section types
- New flags
- New optional fields at end of sections

### Breaking Changes (Major Version Bump)

- Header structure modification
- Section table entry format change
- Core section type redefinition
- Magic number change

---

## Testing Strategy

Following WJTTC (Wolfe-Jam Technical Testing Center) standards:

1. **Correctness** - Round-trip validation
2. **Resilience** - Corruption handling
3. **Performance** - Must be faster than YAML parsing
4. **Edge Cases** - Empty files, max sizes, malformed data
5. **Production Reality** - Real-world .faf files

---

## Status

**Draft v0.1** - Specification only, no implementation yet.

### Milestones

- [ ] Header read/write implementation
- [ ] Section table implementation
- [ ] Core section encoding
- [ ] Round-trip tests passing
- [ ] Performance benchmarks
- [ ] Budget loading
- [ ] Optional features (embeddings, compression)

---

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  SYSTEMS LAYER • WHERE FAF MEETS THE METAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

*Built with F1-inspired engineering principles* 🏎️⚡

*Testing would be paramount. We use what we know and explore.*

---

# PART II: THE EMBEDDINGS LAYER

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  DEEPER • THE AI NATIVE INTERFACE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> "Below the binary format, we hit the real question: How does an AI actually understand FAF?"

---

## ⚠️ TREMOR WARNING

**Tiny changes below are tremors in GUIs - even crashes.**

This is finetuning before sound. Before ANY of this layer:

1. Core binary format MUST be solid
2. Round-trip tests MUST pass
3. Priority truncation MUST be exact
4. Checksum validation MUST be bulletproof

The embeddings layer is OPTIONAL (FLAG_EMBEDDINGS). Systems must work without it.

**Amplification risk:**
```
0.1% embedding error → 10% retrieval error → 50% response failure
```

**Fail-safe requirement:**
- Missing embeddings → fall back to text parsing
- Corrupted embeddings → regenerate from source
- Version mismatch → skip embeddings, use text

---

## The Current Path

```
.faf (YAML) → Tokenizer → Embeddings → Attention → Output
                 ↑            ↑           ↑
            Model does all of this
```

Every time an AI reads a .faf file, it:
1. Tokenizes the text (model-specific)
2. Embeds tokens into vectors
3. Runs attention over embeddings
4. Generates response

This happens fresh every time. Same file, same compute.

---

## The Deeper Path

```
.faf (YAML) → Compile → .fafb (binary)
                           ↓
              Pre-computed embeddings
                           ↓
              Direct injection into model
```

**Pre-compute what doesn't change.**

The project name "faf-rust-sdk" embeds to the same vector every time. Why recompute?

---

## Embedding Architecture

### Layer 1: Section Embeddings

Each section gets a single embedding vector representing its semantic content.

```rust
struct SectionEmbedding {
    section_type: u8,
    dimensions: u16,        // 768, 1024, 1536, etc.
    vector: Vec<f32>,       // The actual embedding
    confidence: f32,        // How stable is this embedding?
}
```

**Use case**: Semantic search across .faf files.

"Find projects similar to this one" becomes vector similarity.

---

### Layer 2: Chunk Embeddings

Finer granularity - embed meaningful chunks within sections.

```rust
struct ChunkEmbedding {
    section_type: u8,
    chunk_index: u16,
    start_offset: u32,      // Byte offset in section data
    end_offset: u32,
    dimensions: u16,
    vector: Vec<f32>,
}
```

**Use case**: Retrieval-augmented generation (RAG).

Model asks "what's the build command?" → vector search → return exact chunk.

---

### Layer 3: Attention Weights

Pre-computed hints for how the model should attend.

```rust
struct AttentionHint {
    section_type: u8,
    weight: f32,            // 0.0 - 1.0, relative importance
    decay_rate: f32,        // How fast importance drops off
    relationship: Vec<(u8, f32)>,  // Cross-section relationships
}
```

**Example**:
```
KEY_FILES.weight = 0.9      // High attention
CONTEXT.weight = 0.3        // Low attention
KEY_FILES → COMMANDS = 0.7  // Strong relationship
```

This is guidance, not control. The model can ignore it.

---

## Model-Agnostic vs Model-Specific

### The Problem

- GPT-4 uses different tokenizer than Claude
- Embedding dimensions differ (1536 vs 768)
- Attention patterns differ

### The Solution: Base + Overlay

```rust
struct EmbeddingsSection {
    // Base embeddings (model-agnostic)
    base_model: String,         // "sentence-transformers/all-MiniLM-L6-v2"
    base_dimensions: u16,       // 384
    base_embeddings: Vec<Vec<f32>>,

    // Model-specific overlays (optional)
    overlays: Vec<ModelOverlay>,
}

struct ModelOverlay {
    model_pattern: String,      // "gpt-4*", "claude-3*", "grok-*"
    dimensions: u16,
    embeddings: Vec<Vec<f32>>,
    token_counts: Vec<u32>,     // Exact token counts for this model
}
```

**Why base embeddings?**

Sentence transformers are open, fast, and universal. Use them for:
- Cross-project search
- Similarity scoring
- Initial retrieval

Model-specific overlays are optional optimizations.

---

## Binary Format for Embeddings Section (0x10)

```
┌─────────────────────────────┐
│  Embeddings Section Header  │
├─────────────────────────────┤
│  base_model_length: u16     │
│  base_model: [u8; len]      │
│  base_dimensions: u16       │
│  embedding_count: u32       │
├─────────────────────────────┤
│  Embedding 0                │
│  ├─ section_type: u8        │
│  ├─ chunk_index: u16        │
│  └─ vector: [f32; dims]     │
├─────────────────────────────┤
│  Embedding 1                │
│  ...                        │
├─────────────────────────────┤
│  Overlay Count: u16         │
├─────────────────────────────┤
│  Overlay 0 (GPT-4)          │
│  ...                        │
└─────────────────────────────┘
```

### Embedding Entry (variable size)

```rust
struct EmbeddingEntry {
    section_type: u8,       // Which section this embeds
    chunk_index: u16,       // 0 = whole section, 1+ = chunk within
    // Followed by: dimensions × f32 values
}
```

Size per embedding: 3 + (dimensions × 4) bytes

For 384-dim embeddings: 3 + 1536 = 1539 bytes per embedding.

---

## Rust Dependencies for Embeddings

```toml
[dependencies]
# Lightweight option (ONNX-based)
fastembed = "0.2"

# Or full power (requires libtorch)
rust-bert = "0.21"
tch = "0.13"

# Efficient array operations
ndarray = "0.15"
```

**Recommendation**: Start with `fastembed` - lighter, portable, good enough for base embeddings.

---

## Breakage Risks at Embeddings Layer

1. **Model drift** - Embedding models get updated, vectors change
2. **Dimension mismatch** - Can't mix 768 and 1536 dim vectors
3. **Semantic shift** - Same text, different meaning in new model
4. **Storage cost** - Embeddings are big (N × dimensions × 4 bytes)
5. **Generation cost** - Need GPU or significant CPU for embedding

**Mitigations**:
- Version the embedding model in header
- Support re-generation from source .faf
- Compress embeddings (quantization to int8)
- Make embeddings optional (FLAG_EMBEDDINGS)
- ALWAYS maintain fallback to text

---

# PART III: THE ATTENTION LAYER

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  DEEPEST • ATTENTION GUIDANCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> "Beyond embeddings - guiding where the model looks."

## ⚠️ TREMOR WARNING - SEVERE

Attention weights directly affect model behavior. Errors here are HIGH IMPACT.

**Safeguards**:
- All weights MUST be 0.0 - 1.0
- Invalid weights → use defaults
- Missing attention section → system works normally
- This is GUIDANCE, not CONTROL

---

## Attention Weights Section (0x12)

```rust
struct AttentionSection {
    project_weight: f32,        // Overall importance
    section_weights: Vec<SectionWeight>,
    relationships: Vec<Relationship>,
}

struct SectionWeight {
    section_type: u8,
    base_weight: f32,           // 0.0 - 1.0
    decay_rate: f32,            // How fast it fades
    boost_on_query: Vec<String>, // Keywords that boost this section
}

struct Relationship {
    source: u8,
    target: u8,
    strength: f32,              // -1.0 to 1.0
}
```

---

## Default Attention Profile

Based on analysis of successful AI interactions:

```rust
const DEFAULT_ATTENTION: &[SectionWeight] = &[
    SectionWeight {
        section_type: SECTION_META,
        base_weight: 1.0,
        decay_rate: 0.0,        // Never decays
        boost_on_query: vec![],
    },
    SectionWeight {
        section_type: SECTION_KEY_FILES,
        base_weight: 0.9,
        decay_rate: 0.1,
        boost_on_query: vec!["where", "file", "find", "location"],
    },
    SectionWeight {
        section_type: SECTION_COMMANDS,
        base_weight: 0.85,
        decay_rate: 0.05,
        boost_on_query: vec!["run", "build", "test", "start", "how"],
    },
];
```

---

# PART IV: THE COMPLETE STACK

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ARCHITECTURE • BASE TO TOP
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

```
┌─────────────────────────────────────────┐
│  .faf (YAML)                            │  Human readable - SOURCE OF TRUTH
├─────────────────────────────────────────┤
│  .fafb Header (32 bytes)                │  File identification
├─────────────────────────────────────────┤
│  Section Data                           │  Content - MUST WORK
├─────────────────────────────────────────┤
│  Section Table                          │  Index - MUST WORK
├─────────────────────────────────────────┤
│  Embeddings Section (OPTIONAL)          │  Semantic - CAN FAIL GRACEFULLY
├─────────────────────────────────────────┤
│  Attention Section (OPTIONAL)           │  Guidance - CAN FAIL GRACEFULLY
└─────────────────────────────────────────┘
```

**The rule**: Everything above the line MUST work. Everything below CAN fail gracefully.

---

## Implementation Order

**Sound before finetuning:**

1. **Phase 1: Core (THE SOUND)**
   - Header read/write
   - Section table
   - Core section encoding
   - Round-trip tests
   - Corruption detection

2. **Phase 2: Smart Loading**
   - Priority truncation
   - Budget loading
   - Performance benchmarks

3. **Phase 3: Embeddings (FINETUNING)**
   - Only after Phase 1+2 are SOLID
   - With full fallback to Phase 1+2
   - Versioned and regenerable

4. **Phase 4: Attention (MORE FINETUNING)**
   - Only after Phase 3 works
   - With full fallback to defaults
   - Validated and bounded

---

## Estimated File Sizes

| Configuration | Size |
|---------------|------|
| Core only | ~2-5 KB |
| + Embeddings (384d) | ~10-20 KB |
| + Attention | +~1 KB |

Still tiny. Even fully loaded under 50KB.

---

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  SYSTEMS LAYER • WHERE FAF MEETS THE METAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
         GET THE SOUND RIGHT FIRST
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

*Built with F1-inspired engineering principles* 🏎️⚡

*Testing would be paramount. We use what we know and explore.*

*Tiny changes below are tremors above. Sound before finetuning.*

*YOLO from the base up. November 2025.*

---

# PART V: CONSUMPTION PATTERNS

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  HOW AI SYSTEMS ACTUALLY CONSUME .fafb
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> "The lowest level question: How does xAI/Grok actually read this?"

---

## The Current Reality (YAML)

When Grok/Claude/GPT reads a .faf file today:

```
User prompt: "Help me build a feature"
     ↓
System prompt includes .faf content (YAML text)
     ↓
Tokenizer splits YAML into tokens
     ↓
Model embeds tokens → vectors
     ↓
Attention runs over embeddings
     ↓
Response generated
```

**Token cost example** (project.faf):

```yaml
faf_version: 2.5.0
project:
  name: faf-rust-sdk
  goal: High-performance Rust SDK
instant_context:
  tech_stack: Rust 2021, serde
  key_files:
    - src/lib.rs
    - src/parser.rs
```

**Tokenization** (GPT-4 tokenizer):
- `faf_version` = 3 tokens
- `project:` = 2 tokens
- Indentation = 1 token each
- Total: ~90 tokens for this snippet

Every. Single. Time.

---

## Path 1: Pre-Tokenized .fafb

Skip the tokenizer entirely. Include token boundaries in binary.

```rust
struct TokenizedSection {
    section_type: u8,
    token_count: u32,
    tokens: Vec<u32>,           // Token IDs (model-specific)
}
```

**Consumption**:
```
Load .fafb → Read token IDs → Direct to embedding layer
```

**Savings**: Tokenizer pass eliminated (~5-10ms per load)

**Problem**: Model-specific. GPT-4 tokens ≠ Claude tokens ≠ Grok tokens.

**Mitigation**: Multiple token overlays per model family:
```rust
struct TokenOverlay {
    model_pattern: String,      // "gpt-4*", "claude-*", "grok-*"
    tokens: Vec<u32>,
}
```

---

## Path 2: Pre-Embedded .fafb

Skip tokenizer AND embedding. Include vectors directly.

```
Load .fafb → Read vectors → Direct to attention layer
```

**Savings**: Tokenizer + embedding pass eliminated

**Problem**: Even MORE model-specific. Embedding spaces differ.

**Reality check**: This is aggressive. But for HIGH-FREQUENCY contexts (project identity, tech stack), could be worth it.

---

## Path 3: Optimized Text

Keep it text, but optimize for token efficiency.

**Before** (~90 tokens):
```yaml
faf_version: 2.5.0
project:
  name: faf-rust-sdk
  goal: High-performance Rust SDK
instant_context:
  tech_stack: Rust 2021, serde
```

**After** (~45 tokens):
```
[FAF:2.5.0|faf-rust-sdk|Rust SDK]
stack:Rust2021,serde
files:lib.rs,parser.rs
```

**50% token reduction** with same information density.

This could be a "compact" output mode for .fafb → text conversion.

---

## xAI/Grok Specific Considerations

Grok's architecture (based on public info):

1. **X integration** - Context from conversations, posts, threads
2. **Real-time training** - More recent knowledge than other models
3. **Personality** - More direct/irreverent responses

**How .fafb helps Grok**:

1. **Structured context injection**
   - Grok gets project DNA in system prompt
   - Pre-formatted for token efficiency
   - Priority system for context window management

2. **X thread context**
   - Tweet about faf-rust-sdk
   - Grok knows what faf-rust-sdk IS (from .faf)
   - Can reference key_files, tech_stack, commands

3. **Consistency**
   - Same project context across conversations
   - No re-explanation needed

---

## Claude MCP Consumption

Claude's MCP (Model Context Protocol) architecture:

```
┌─────────────────┐
│  Claude Desktop │
├─────────────────┤
│  MCP Server     │ ← .fafb loaded here
├─────────────────┤
│  Tools/Context  │
└─────────────────┘
```

**claude-faf-mcp** (existing):
- Loads .faf files via MCP
- Provides `faf_context` tool
- Returns structured project DNA

**claude-faf-mcp + .fafb**:
- Same API surface
- Backend uses .fafb for faster loading
- Token budgets respected via priority system

```rust
// MCP tool handler
async fn faf_context(request: FafRequest) -> FafResponse {
    // Old: parse YAML every time
    // let faf = parse_faf(&path)?;

    // New: load binary with budget
    let faf = FafBinary::load_with_budget(&path, request.token_budget)?;

    // Same response format
    FafResponse { context: faf.to_mcp_format() }
}
```

---

## RAG Pipeline Integration

For retrieval-augmented generation:

```
User query
    ↓
Embed query → vector
    ↓
Search .fafb embeddings (if FLAG_EMBEDDINGS)
    ↓
Retrieve relevant chunks
    ↓
Inject into context
    ↓
Generate response
```

**Use case**: "Find projects that use Rust and have MCP integration"

1. Embed query
2. Search across all .fafb files in index
3. Return top matches (based on section embeddings)
4. Load full context for top 3

---

## Streaming Consumption

For large .fafb files or constrained memory:

```rust
impl FafBinary {
    /// Stream sections by priority
    pub fn stream_sections(&self) -> impl Iterator<Item = Section> {
        self.sections
            .iter()
            .sorted_by_key(|s| Reverse(s.priority))
            .map(|entry| self.load_section(entry))
    }
}
```

Consumer can stop when budget exhausted:

```rust
let mut budget = 4096;
for section in fafb.stream_sections() {
    if section.token_count > budget {
        break;
    }
    context.push(section);
    budget -= section.token_count;
}
```

---

## Multi-Model Distribution

Scenario: Same project used across Grok, Claude, GPT

```
                    ┌─────────┐
project.faf ──────► │ Compile │
                    └────┬────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
         project.fafb  project.fafb  project.fafb
         (Grok hints)  (Claude hints) (GPT hints)
```

Or single file with multiple overlays:

```rust
struct FafBinaryMulti {
    core: FafBinaryCore,                    // Universal
    overlays: HashMap<String, ModelOverlay>, // Per-model
}
```

Single .fafb file, multiple consumption patterns.

---

## Token Budget Examples

Real-world scenarios:

| Model | Context Window | .faf Budget | Strategy |
|-------|---------------|-------------|----------|
| GPT-4o | 128K | 2000-4000 | Full load |
| Claude 3.5 | 200K | 4000-8000 | Full load |
| Grok | ~8K? | 500-1000 | Priority truncation |
| GPT-4-mini | 16K | 800-1200 | Standard compression |

The priority system handles this automatically:

```rust
// Grok with small window
let context = FafBinary::load_with_budget(&data, 500)?;
// Gets: META + TECH_STACK + KEY_FILES (top 3-5)

// Claude with large window
let context = FafBinary::load_with_budget(&data, 8000)?;
// Gets: Everything
```

---

## IDE Integration

How editors consume .fafb:

**Cursor/VSCode extension**:
```typescript
const fafb = await loadFafBinary(workspace.root);

// Quick access for file hover
const keyFiles = fafb.getSection(SECTION_KEY_FILES);

// Commands palette
const commands = fafb.getSection(SECTION_COMMANDS);

// Full context for AI chat
const context = fafb.toSystemPrompt(budgetTokens);
```

**Benefits**:
- Instant section access (O(1) lookup)
- No YAML parsing on every keystroke
- Token-budgeted context injection

---

## Caching Strategy

.fafb enables smart caching:

```rust
struct FafCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

struct CacheEntry {
    checksum: u32,              // From .fafb header
    parsed: FafBinary,
    embeddings: Option<Vec<Vec<f32>>>,
    last_accessed: Instant,
}

impl FafCache {
    fn get(&mut self, path: &Path, new_checksum: u32) -> &FafBinary {
        if let Some(entry) = self.entries.get(path) {
            if entry.checksum == new_checksum {
                // Cache hit - .faf unchanged
                return &entry.parsed;
            }
        }
        // Cache miss - reload
        self.reload(path)
    }
}
```

Checksum in header enables instant staleness detection without reading full file.

---

## Consumption Benchmarks (Theoretical)

| Operation | YAML | Binary | Speedup |
|-----------|------|--------|---------|
| Full parse | 5ms | 0.5ms | 10x |
| Section lookup | 3ms | 0.01ms | 300x |
| Token estimate | 10ms | 0ms (pre-computed) | ∞ |
| Truncation | 8ms | 0.1ms | 80x |
| Checksum validation | N/A | 0.001ms | - |

These are estimates. Actual benchmarks needed during implementation.

---

## Edge Cases

**Empty .faf**:
- Minimum viable: META section only
- ~50 bytes compiled

**Huge .faf** (1000 key_files):
- Binary advantage increases with size
- Streaming load prevents OOM

**Rapid updates**:
- Checksum detects changes instantly
- Re-compile only on modification

**Corrupt network transfer**:
- CRC32 catches it immediately
- Fail-safe to source .faf

---

## What We Don't Know Yet

Questions for real-world testing:

1. **Actual token counts** - How accurate are estimates vs reality?
2. **Embedding model choice** - Which base model works best?
3. **Compression trade-offs** - When does zstd help vs hurt?
4. **Cross-platform** - Does little-endian assumption hold?
5. **Memory mapping** - Practical benefits on real workloads?

These need data. Spec first, measure second, optimize third.

---

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  CONSUMPTION • HOW AI ACTUALLY USES THIS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
         GET IT IN THEIR HANDS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

*Built with F1-inspired engineering principles* 🏎️⚡

*The best format is the one AI can actually use efficiently.*

*YOLO continues. November 2025.*

---

# PART VI: COMPETITIVE LANDSCAPE & FUTURE

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  WHAT ELSE EXISTS • WHERE WE FIT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> "Know the territory."

---

## Existing Binary AI Formats

### GGUF (GGML Universal Format)

Used by llama.cpp, Ollama, etc.

```
Header → Metadata → Tensors
```

**Similarities to .fafb**:
- Binary format for AI consumption
- Versioned header
- Flexible metadata

**Differences**:
- GGUF is for **models**, .fafb is for **context**
- GGUF stores weights, .fafb stores project DNA
- GGUF is huge (GBs), .fafb is tiny (KBs)

**Relationship**: Complementary. Model in GGUF, context in .fafb.

---

### ONNX (Open Neural Network Exchange)

Industry standard for model interchange.

**Similarities**:
- Cross-platform format
- Version controlled
- Standardized through consortium

**Differences**:
- ONNX is compute graphs, .fafb is structured data
- Different consumers (runtime vs model)

**Lesson learned**: ONNX succeeded via industry consortium (Microsoft, Facebook, AWS). FAF Foundation could follow similar path.

---

### Safetensors (Hugging Face)

Fast, safe tensor serialization.

**Similarities**:
- Zero-copy loading
- Memory mapped
- Size-efficient

**Differences**:
- Tensors vs structured context
- Different scale (GBs vs KBs)

**Lesson learned**: Safetensors won by being **faster and safer** than pickle. Performance matters.

---

### Protobuf / FlatBuffers

Google's serialization formats.

**Similarities**:
- Binary efficiency
- Schema evolution
- Cross-language support

**Differences**:
- General purpose vs AI-context specific
- No semantic layers (embeddings, attention)

**Consideration**: Could .fafb use Protobuf/FlatBuffers internally?
- Pro: Proven, tooling exists
- Con: Adds dependency, less control over format

**Decision**: Start custom for flexibility, consider migration later if needed.

---

## Emerging AI Context Formats

### Cursor's .cursorrules

Project-specific rules for Cursor IDE.

**Format**: Markdown/text
**Purpose**: Customize AI behavior per project

**Comparison**:
- .cursorrules is Cursor-specific, .faf is universal
- .faf has richer structure (tech stack, key files, commands)
- Both aim to reduce AI context setup

**Integration opportunity**: .faf could export to .cursorrules format.

---

### .windsurfrules (Codeium)

Similar concept for Windsurf editor.

**Pattern**: Every AI IDE is creating its own context format.

**FAF's value**: Universal format that all of these can import from.

```
project.faf (source of truth)
    ├──► .cursorrules
    ├──► .windsurfrules
    ├──► .github/copilot-instructions.md
    └──► system prompt (any model)
```

---

### LLMS.txt

Proposed standard for AI-readable website metadata.

**Format**: Text file at website root
**Purpose**: Tell LLMs what to know about a site

**Comparison**:
- LLMS.txt is for websites, .faf is for projects
- Different scopes but similar philosophy

**Note**: LLMS.txt shows appetite for AI-context standards.

---

## Where .fafb Fits

```
┌───────────────────────────────────────────────┐
│  Model Formats (GGUF, ONNX, Safetensors)      │  GBs - The AI
├───────────────────────────────────────────────┤
│  Context Formats (.fafb)                      │  KBs - What AI knows
├───────────────────────────────────────────────┤
│  IDE Formats (.cursorrules, .windsurfrules)   │  Text - How AI behaves
└───────────────────────────────────────────────┘
```

.fafb is the middle layer - structured project knowledge.

---

## Future Directions

### 1. Federation

Multiple .faf files linked together:

```yaml
# project.faf
federation:
  - https://faf.one/standards/rust
  - ./local-overrides.faf
```

Binary format includes federation table:
```rust
struct FederationEntry {
    source_uri: String,
    checksum: u32,
    priority: u8,      // Override order
}
```

**Use case**: Organization-wide standards + project-specific overrides.

---

### 2. Streaming Updates

Real-time context updates without full reload:

```rust
struct FafbPatch {
    version: u32,
    operations: Vec<PatchOp>,
}

enum PatchOp {
    UpdateSection { section_type: u8, data: Vec<u8> },
    AddSection { entry: SectionEntry, data: Vec<u8> },
    RemoveSection { section_type: u8 },
    UpdateEmbedding { index: u32, vector: Vec<f32> },
}
```

**Use case**: Live project state without re-compilation.

---

### 3. Signed Contexts

Cryptographic verification of context authenticity:

```rust
struct SignedFafb {
    fafb_data: Vec<u8>,
    signature: [u8; 64],      // Ed25519
    public_key: [u8; 32],
    timestamp: u64,
}
```

**Use case**: Enterprise contexts that must be tamper-proof.

---

### 4. Differential Loading

Load only changed sections since last checkpoint:

```rust
impl FafBinary {
    fn load_differential(
        &self,
        last_checksum: u32,
        changed_sections: &[u8],
    ) -> DiffResult {
        // Only load sections that changed
    }
}
```

**Use case**: CI/CD where context changes incrementally.

---

### 5. Cross-Language SDKs

Current: Rust, Python, TypeScript
Future: Go, Java, Swift, C++, WASM

Binary format enables fast native implementations:

```go
// Go SDK
fafb, err := faf.LoadBinary("project.fafb")
if err != nil {
    return err
}
fmt.Println(fafb.ProjectName())
```

**Priority**: Rust (done) → Python → Go → Java

---

### 6. Registry Integration

Publish .fafb to package registries:

```bash
# Publish to faf.one registry
faf publish project.fafb

# Install as dependency
faf install @org/shared-context
```

Binary format makes this efficient (small, versioned, checksummed).

---

### 7. AI Model Fine-tuning

Use .fafb corpus for model training:

1. Collect .fafb files from open source projects
2. Extract embeddings and attention patterns
3. Fine-tune model on "good" project context
4. Model learns what makes context effective

**Research question**: Can models learn FAF format natively?

---

## Standards Path

### IANA Registration (Done)

- `application/vnd.faf+yaml` registered October 2025
- Establishes FAF as legitimate format

### Future Registrations

- `application/vnd.fafb` for binary format
- Or `application/vnd.fafb`

### W3C / ECMA Consideration

If adoption grows:
1. Form FAF Foundation (Rust Foundation model)
2. Submit to standards body
3. Multi-stakeholder governance

**Timeline**: Years, not months. Organic adoption first.

---

## Risks and Mitigations

### Risk 1: Format Fragmentation

Each AI vendor creates incompatible format.

**Mitigation**:
- Open spec, permissive license
- Reference implementations in multiple languages
- Export adapters for vendor formats

---

### Risk 2: Embedding Model Lock-in

Everyone uses different embedding models.

**Mitigation**:
- Base embeddings with universal model
- Model-specific overlays optional
- Regeneration from source always possible

---

### Risk 3: Complexity Creep

Spec grows too complex to implement.

**Mitigation**:
- Core spec is small (Parts I-II)
- Everything else is OPTIONAL flags
- Conformance levels (minimal, standard, full)

---

### Risk 4: No Adoption

Developers don't use it.

**Mitigation**:
- Zero setup (init command)
- Immediate value (better AI responses)
- Works with existing tools
- Open source, free

---

## Success Metrics

How we know .fafb is working:

1. **SDK Downloads**: crates.io, PyPI, npm
2. **GitHub Stars**: Organic growth
3. **Integration Count**: Tools that support .faf/.fafb
4. **Token Savings**: Measured reduction in context costs
5. **Response Quality**: User reports of better AI outputs

---

## The Long Game

```
2025: Format established (.faf)
      ↓
      Binary format spec'd (.fafb)
      ↓
2026: Multi-language SDKs
      ↓
      IDE integrations
      ↓
      Model-specific optimizations
      ↓
2027: Federation & signing
      ↓
      Standards body consideration
      ↓
????: Native model support
```

This is infrastructure. Infrastructure takes time.

---

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  LANDSCAPE • KNOW THE TERRITORY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
         BUILD FOR THE LONG GAME
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

*Built with F1-inspired engineering principles* 🏎️⚡

*Standards win through adoption, not declaration.*

*The spec is done when it's useful, not when it's complete.*

*YOLO complete. November 2025.*

---

# APPENDIX A: QUICK REFERENCE

## File Identification

```
Magic: 0x46 0x41 0x46 0x42 ("FAFB")
Extension: .fafb
MIME (proposed): application/vnd.fafb
```

## Minimum Valid .fafb

```
Header (32 bytes)
+ META section (variable)
+ Section table (16 bytes)
= ~50-100 bytes minimum
```

## Priority Cheat Sheet

| Priority | Value | Never Truncate? |
|----------|-------|-----------------|
| Critical | 255 | Yes |
| High | 200 | Rarely |
| Medium | 128 | Normal |
| Low | 64 | Often |
| Optional | 0 | Always OK |

## Flag Bits

```
Bit 0: Compressed (zstd)
Bit 1: Embeddings present
Bit 2: Tokenized
Bit 3: Attention weights
Bit 4: Model hints
Bit 5: Signed
Bits 6-15: Reserved
```

## Implementation Checklist

- [ ] Header read/write
- [ ] Section table read/write
- [ ] META section encoding
- [ ] TECH_STACK section encoding
- [ ] KEY_FILES section encoding
- [ ] COMMANDS section encoding
- [ ] Round-trip test passing
- [ ] Corruption detection
- [ ] Priority truncation
- [ ] Budget loading
- [ ] Performance benchmarks
- [ ] Embeddings (optional)
- [ ] Attention (optional)

---

*End of FAFB Binary Format Specification v0.1*

*This is a living document. The spec evolves with implementation learnings.*
