# Phase 1: Core Types and Parser - Research

**Researched:** 2026-02-04
**Domain:** Rust types, builder patterns, hash-based identifiers, and parser combinators
**Confidence:** HIGH

## Summary

This phase establishes the foundational types (`Phrase`, `Value`, `PhraseId`) and the parser for RLF's template syntax and `.rlf` file format. The research confirms a standard stack using `bon` for builder patterns, `thiserror` for error handling, `winnow` for parsing, and `const-fnv1a-hash` for compile-time phrase ID hashing. The design documents (DESIGN.md, APPENDIX_RUST_INTEGRATION.md) provide comprehensive specifications that should be followed precisely.

Key insights: The parser must handle the complete RLF grammar including interpolations, transforms, selections, escape sequences, and phrase calls. The `PhraseId` type uses FNV-1a hashing and must be `const fn` constructible. Newtypes `VariantKey` and `Tag` provide type safety over raw strings.

**Primary recommendation:** Follow the design documents exactly; use `bon` for builders, `winnow` for parsing, and implement `const fn` FNV-1a hashing for `PhraseId` using the `const-fnv1a-hash` crate or inline implementation.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| [bon](https://docs.rs/bon/latest/bon/) | 3.8.x | Builder pattern derive macro | Compile-time checked, zero panics, order-independent setters |
| [thiserror](https://docs.rs/thiserror/latest/thiserror/) | 2.0.x | Error type derive macro | Standard for Display + std::error::Error impl |
| [winnow](https://docs.rs/winnow/latest/winnow/) | 0.7.x | Parser combinator library | Successor to nom, better error handling, modern API |
| [serde](https://serde.rs/) | 1.x | Serialization | Required for `PhraseId` serialization (TYPE-11) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| [const-fnv1a-hash](https://docs.rs/const-fnv1a-hash/latest/const_fnv1a_hash/) | 1.x | Const FNV-1a hashing | For `PhraseId::from_name()` const fn |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| winnow | nom v8 | nom is more established but winnow has better errors and is nom's spiritual successor |
| bon | typed-builder | bon has more features and is actively maintained |
| thiserror | anyhow | thiserror is for library errors; anyhow is for applications |

**Installation:**
```bash
cargo add bon thiserror winnow serde --features serde/derive
cargo add const-fnv1a-hash
```

Or in Cargo.toml:
```toml
[dependencies]
bon = "3.8"
thiserror = "2.0"
winnow = "0.7"
serde = { version = "1.0", features = ["derive"] }
const-fnv1a-hash = "1.1"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── lib.rs           # Public API exports
├── types/
│   ├── mod.rs       # Re-exports
│   ├── phrase.rs    # Phrase struct
│   ├── value.rs     # Value enum
│   ├── phrase_id.rs # PhraseId hash wrapper
│   ├── variant_key.rs # VariantKey newtype
│   └── tag.rs       # Tag newtype
├── parser/
│   ├── mod.rs       # Parser public API
│   ├── template.rs  # Template string parser
│   ├── file.rs      # .rlf file format parser
│   ├── ast.rs       # AST types (PhraseDefinition, Segment, etc.)
│   └── error.rs     # Parse error types
└── error.rs         # EvalError, LoadError types
```

### Pattern 1: Newtype with Deref
**What:** Wrap primitive types in newtypes for type safety
**When to use:** For `VariantKey` and `Tag` which wrap String
**Example:**
```rust
// Source: Rust Design Patterns - Newtype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantKey(String);

impl VariantKey {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for VariantKey {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for VariantKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
```

### Pattern 2: Builder with bon
**What:** Use bon derive for complex struct construction
**When to use:** For `Phrase` struct with optional fields
**Example:**
```rust
// Source: https://docs.rs/bon/latest/bon/
use bon::Builder;

#[derive(Debug, Clone, Builder)]
pub struct Phrase {
    pub text: String,
    #[builder(default)]
    pub variants: HashMap<VariantKey, String>,
    #[builder(default)]
    pub tags: Vec<Tag>,
}

// Usage:
let phrase = Phrase::builder()
    .text("card".to_string())
    .variants(HashMap::from([(VariantKey::new("one"), "card".into())]))
    .build();
```

### Pattern 3: Const fn Hash Construction
**What:** PhraseId from name as const fn for compile-time constants
**When to use:** For `PhraseId::from_name()`
**Example:**
```rust
// Source: https://docs.rs/const-fnv1a-hash/latest/const_fnv1a_hash/
use const_fnv1a_hash::fnv1a_hash_str_64;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PhraseId(u64);

impl PhraseId {
    pub const fn from_name(name: &str) -> Self {
        Self(fnv1a_hash_str_64(name, None))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Compile-time constant:
const CARD_ID: PhraseId = PhraseId::from_name("card");
```

### Pattern 4: Parser Combinator Structure
**What:** Organize winnow parsers as functions returning Parser trait
**When to use:** For all parsing functions
**Example:**
```rust
// Source: https://docs.rs/winnow/latest/winnow/
use winnow::prelude::*;
use winnow::combinator::{seq, alt, repeat};
use winnow::token::{take_while, take_until};

fn identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .parse_next(input)
}

fn interpolation<'i>(input: &mut &'i str) -> ModalResult<Segment> {
    seq!(
        _: '{',
        transforms: repeat(0.., transform),
        reference: reference,
        selectors: repeat(0.., selector),
        _: '}'
    ).map(|(transforms, reference, selectors)| {
        Segment::Interpolation { transforms, reference, selectors }
    }).parse_next(input)
}
```

### Anti-Patterns to Avoid
- **Hand-rolling FNV-1a hash:** Use const-fnv1a-hash crate for correctness and `const fn` support
- **Using raw String for variant keys:** Always use `VariantKey` newtype for type safety
- **Exposing parser internals:** Keep AST types public but parser implementation private
- **Left-recursive grammar:** Can cause stack overflow; use iteration or left-factoring

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Builder pattern | Manual builder struct | `bon::Builder` derive | Compile-time checking, zero panics |
| Error Display impl | Manual Display impl | `thiserror::Error` derive | Automatic Display from #[error(...)] |
| FNV-1a hashing | Manual hash loop | `const-fnv1a-hash` | Const fn support, verified correctness |
| Parser combinators | Manual char iteration | `winnow` | Error handling, composability, backtracking |
| Serde derives | Manual Serialize/Deserialize | `serde` derive | Standard ecosystem compatibility |

**Key insight:** The Rust ecosystem has mature, well-tested solutions for all common patterns in this phase. Custom implementations are likely to have edge case bugs that established crates have already fixed.

## Common Pitfalls

### Pitfall 1: Escape Sequence Handling
**What goes wrong:** Parser doesn't correctly handle `{{`, `}}`, `@@`, `::` escapes
**Why it happens:** Tempting to parse `{` as always starting interpolation
**How to avoid:** Parse escape sequences first in alt(), before non-escaped forms
**Warning signs:** Test with "Use {{name}} syntax" fails to produce literal braces

### Pitfall 2: Unicode in Identifiers
**What goes wrong:** Parser rejects valid UTF-8 phrase names or crashes on multibyte
**Why it happens:** Using byte offsets instead of character boundaries
**How to avoid:** winnow handles UTF-8 correctly; use `char` predicates not byte predicates
**Warning signs:** Errors mention "invalid character" for valid Unicode

### Pitfall 3: Variant Key Fallback Order
**What goes wrong:** `nom.one` doesn't fall back to `nom` correctly
**Why it happens:** HashMap lookup is exact match only
**How to avoid:** Implement fallback resolution: exact -> drop last segment -> repeat
**Warning signs:** MissingVariant error when fallback should have matched

### Pitfall 4: PhraseId Collision Handling
**What goes wrong:** Two different phrase names hash to same u64
**Why it happens:** 64-bit hash collisions are rare but possible
**How to avoid:** Detect at registration time, panic with clear message
**Warning signs:** Silent wrong phrase returned; only visible with many phrases

### Pitfall 5: Parser Error Spans
**What goes wrong:** Error says "line 5" but points to wrong location
**Why it happens:** Line counting off-by-one or not tracking position properly
**How to avoid:** Use winnow's built-in position tracking; test error positions
**Warning signs:** User confusion about error locations

### Pitfall 6: Transform Context vs Selection
**What goes wrong:** `@der:acc` parsed as selection instead of transform context
**Why it happens:** Both use `:` syntax; order matters
**How to avoid:** Parse transforms with optional context BEFORE reference+selectors
**Warning signs:** German case transforms produce parse errors

## Code Examples

Verified patterns from official sources:

### Phrase Struct with Builder
```rust
// Source: DESIGN.md, APPENDIX_RUST_INTEGRATION.md
use bon::Builder;
use std::collections::HashMap;

#[derive(Debug, Clone, Builder)]
pub struct Phrase {
    /// Default text when displayed
    pub text: String,
    /// Variant key -> variant text
    #[builder(default)]
    pub variants: HashMap<VariantKey, String>,
    /// Metadata tags
    #[builder(default)]
    pub tags: Vec<Tag>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution.
    /// Tries exact match first, then progressively shorter keys.
    /// Panics if no match found.
    pub fn variant(&self, key: &str) -> &str {
        // Try exact match
        if let Some(v) = self.variants.get(&VariantKey::new(key)) {
            return v;
        }
        // Try progressively shorter keys
        let mut current = key;
        while let Some(dot_pos) = current.rfind('.') {
            current = &current[..dot_pos];
            if let Some(v) = self.variants.get(&VariantKey::new(current)) {
                return v;
            }
        }
        // No match - panic with helpful error
        panic!(
            "No variant '{}' in phrase. Available: {:?}",
            key,
            self.variants.keys().collect::<Vec<_>>()
        );
    }
}

impl std::fmt::Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

### Value Enum with Into Implementations
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(Phrase),
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as i64)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Float(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Phrase> for Value {
    fn from(p: Phrase) -> Self {
        Value::Phrase(p)
    }
}
```

### PhraseId with Serde
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
use const_fnv1a_hash::fnv1a_hash_str_64;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhraseId(u64);

impl PhraseId {
    /// Create a PhraseId from a phrase name at compile time.
    pub const fn from_name(name: &str) -> Self {
        Self(fnv1a_hash_str_64(name, None))
    }

    /// Get the raw hash value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Note: resolve() and call() methods require Locale which is Phase 4
// name() method requires a registry which is built during phrase loading
```

### Parser AST Types
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
#[derive(Debug, Clone)]
pub struct PhraseDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub tags: Vec<Tag>,
    pub from_param: Option<String>,  // For :from(param)
    pub body: PhraseBody,
}

#[derive(Debug, Clone)]
pub enum PhraseBody {
    Simple(Template),
    Variants(Vec<(Vec<VariantKey>, Template)>),  // Multi-key support
}

#[derive(Debug, Clone)]
pub struct Template {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub enum Segment {
    Literal(String),
    Interpolation {
        transforms: Vec<Transform>,
        reference: Reference,
        selectors: Vec<Selector>,
    },
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub name: String,
    pub context: Option<Selector>,
}

#[derive(Debug, Clone)]
pub enum Reference {
    Parameter(String),
    Phrase(String),
    PhraseCall { name: String, args: Vec<Reference> },
}

#[derive(Debug, Clone)]
pub enum Selector {
    Literal(String),
    Parameter(String),
}
```

### Parse Error Type
```rust
// Source: User decision - thiserror with line:column
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("syntax error at {line}:{column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("unexpected end of input at {line}:{column}")]
    UnexpectedEof {
        line: usize,
        column: usize,
    },

    #[error("invalid UTF-8 in input")]
    InvalidUtf8,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| nom parsers | winnow parsers | 2023 | Better error messages, cleaner API |
| typed-builder | bon | 2024 | More features, compile-time checking |
| Manual hash impl | const-fnv1a-hash | Ongoing | Guaranteed const fn correctness |
| thiserror 1.x | thiserror 2.x | 2024 | Minor API improvements |

**Deprecated/outdated:**
- **nom v7 APIs**: winnow has superseded most nom patterns
- **Manual builder patterns**: bon/typed-builder are now standard

## Open Questions

Things that couldn't be fully resolved:

1. **Winnow Error Context Strategy**
   - What we know: Winnow has multiple error handling modes
   - What's unclear: Best approach for RLF's error requirements (line:column, brief explanation)
   - Recommendation: Start with Winnow's default errors, refine during implementation

2. **Public vs Private AST**
   - What we know: User decided AST types are public for tooling
   - What's unclear: Exact boundary of what's public vs internal
   - Recommendation: Make all AST types in `ast.rs` public, keep parser functions private

## Sources

### Primary (HIGH confidence)
- DESIGN.md - Complete RLF syntax specification
- APPENDIX_RUST_INTEGRATION.md - Rust API design and type specifications
- [bon docs](https://docs.rs/bon/latest/bon/) - Builder pattern API (v3.8.2)
- [thiserror docs](https://docs.rs/thiserror/latest/thiserror/) - Error handling API (v2.0.18)
- [winnow docs](https://docs.rs/winnow/latest/winnow/) - Parser combinator API (v0.7.14)
- [const-fnv1a-hash docs](https://docs.rs/const-fnv1a-hash/latest/const_fnv1a_hash/) - FNV-1a const fn

### Secondary (MEDIUM confidence)
- [Rust Design Patterns - Newtype](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html) - Newtype pattern best practices
- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html) - API design guidelines

### Tertiary (LOW confidence)
- WebSearch results on parser pitfalls - General guidance, verify in practice

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified on docs.rs with current versions
- Architecture: HIGH - Based on design documents and standard Rust patterns
- Pitfalls: MEDIUM - Based on general parser experience, verify in implementation

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (stable domain, 30 days)
