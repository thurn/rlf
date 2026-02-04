# Phase 3: Universal Transforms and ICU4X - Research

**Researched:** 2026-02-04
**Domain:** Transform execution system with ICU4X locale-sensitive case mapping
**Confidence:** HIGH

## Summary

This phase implements the transform execution system and three universal case transforms (@cap, @upper, @lower). The existing codebase from Phases 1-2 has the infrastructure in place: `TransformRegistry` (currently empty), `Transform` AST type with context support, and the evaluator stub that passes transforms through without processing. Phase 3 fills in these pieces with actual transform execution.

The standard approach uses ICU4X `icu_casemap` for locale-sensitive case operations (critical for Turkish dotted-I handling) and `unicode-segmentation` for grapheme-aware first-character identification (@cap). Transforms receive a `Value`, access to optional context, and return either a transformed `Value` or an `EvalError`. The evaluator processes transforms right-to-left (innermost first) per the design documents.

Key decisions from CONTEXT.md constrain the architecture: static dispatch via enum (no trait objects), transforms receive `Value` and return `String`, error on missing tags returns `EvalError`, and ICU4X handles locale-sensitive case mapping especially for Turkish i/I variants.

**Primary recommendation:** Implement `TransformKind` enum for static dispatch, integrate `icu_casemap` with `CaseMapper::new()` for locale-sensitive operations, use `unicode-segmentation` for grapheme-safe @cap, and wire transforms into the evaluator's interpolation processing.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| icu_casemap | 2.1 | Locale-sensitive case mapping | Official Unicode ICU4X implementation, handles Turkish/Greek correctly |
| unicode-segmentation | 1.12 | Grapheme cluster iteration | UAX#29 compliant, 10M downloads/month, no-std compatible |
| icu_locale_core | 2.1 | Locale parsing (already in project) | Required by icu_casemap for langid! macro |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.0 | Error types (already in project) | EvalError extensions |
| bon | 3.8 | Builder pattern (already in project) | If new structs need builders |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| icu_casemap | str::to_uppercase() | Rust stdlib is NOT locale-aware; fails for Turkish (istanbul -> ISTANBUL not ISTANBUL) |
| unicode-segmentation | str.chars().next() | Doesn't handle combining characters; "e\u{0301}" shows as 2 chars not 1 grapheme |

**Installation:**
```bash
cargo add icu_casemap@2 unicode-segmentation@1.12
```

## Architecture Patterns

### Recommended Project Structure
```
crates/rlf/src/
  interpreter/
    transforms.rs      # TransformKind enum, execute_transform(), TransformRegistry
    evaluator.rs       # Wire transforms into interpolation evaluation
    plural.rs          # (existing) CLDR plural rules
    mod.rs             # Export TransformKind
```

### Pattern 1: Static Dispatch via Enum
**What:** Define `TransformKind` enum with variants for each transform type. Each variant knows how to execute.
**When to use:** All transform dispatch. No trait objects or function pointers per CONTEXT.md decision.
**Example:**
```rust
// Source: CONTEXT.md decisions - static dispatch via enum
pub enum TransformKind {
    // Universal transforms
    Cap,
    Upper,
    Lower,
    // Future: language-specific transforms
    // A,      // English @a/@an
    // Der,    // German @der/@die/@das
}

impl TransformKind {
    pub fn execute(
        &self,
        value: Value,
        context: Option<&Value>,
        lang: &str,
    ) -> Result<String, EvalError> {
        match self {
            TransformKind::Cap => cap_transform(value, lang),
            TransformKind::Upper => upper_transform(value, lang),
            TransformKind::Lower => lower_transform(value, lang),
        }
    }
}
```

### Pattern 2: Transform Lookup by Name and Language
**What:** Map transform names to `TransformKind`, with language family fallback (pt-BR -> pt -> universal).
**When to use:** When evaluator encounters a `Transform` AST node.
**Example:**
```rust
// Source: CONTEXT.md - language family fallback, aliases
impl TransformRegistry {
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformKind> {
        // 1. Try exact language match
        // 2. Try language family (pt-BR -> pt)
        // 3. Try universal transforms

        // Handle aliases at lookup time (parser can also normalize)
        let canonical_name = match name {
            "an" => "a",      // @an -> @a
            "die" | "das" => "der", // @die/@das -> @der
            "la" => "el",     // Spanish @la -> @el
            other => other,
        };

        // Universal transforms always available
        match canonical_name {
            "cap" => Some(TransformKind::Cap),
            "upper" => Some(TransformKind::Upper),
            "lower" => Some(TransformKind::Lower),
            _ => None, // Language-specific transforms added in Phases 6-9
        }
    }
}
```

### Pattern 3: Transform Execution in Evaluator
**What:** Apply transforms right-to-left after reference resolution and selector application.
**When to use:** In `eval_template()` for each interpolation.
**Example:**
```rust
// Source: DESIGN.md - transforms execute right-to-left
// "{@cap @a card}" -> @a first, then @cap
fn apply_transforms(
    mut value: Value,
    transforms: &[Transform],
    registry: &TransformRegistry,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<String, EvalError> {
    // Process right-to-left (reverse iteration)
    for transform in transforms.iter().rev() {
        let transform_kind = registry
            .get(&transform.name, lang)
            .ok_or_else(|| EvalError::UnknownTransform {
                name: transform.name.clone(),
            })?;

        // Resolve context if present
        let context_value = if let Some(ctx_selector) = &transform.context {
            Some(resolve_context(ctx_selector, ctx, lang)?)
        } else {
            None
        };

        let result_str = transform_kind.execute(value, context_value.as_ref(), lang)?;
        value = Value::String(result_str);
    }

    Ok(value.to_string())
}
```

### Pattern 4: Grapheme-Aware Capitalization
**What:** Use `unicode-segmentation` to find first grapheme, capitalize it with ICU4X.
**When to use:** @cap transform implementation.
**Example:**
```rust
// Source: unicode-segmentation docs + ICU4X CaseMapper
use unicode_segmentation::UnicodeSegmentation;
use icu_casemap::CaseMapper;
use icu_locale_core::langid;

fn cap_transform(value: Value, lang: &str) -> Result<String, EvalError> {
    let text = value.to_string();
    if text.is_empty() {
        return Ok(text);
    }

    let cm = CaseMapper::new();
    let locale = parse_langid(lang);

    // Get first grapheme cluster
    let mut graphemes = text.graphemes(true);
    if let Some(first) = graphemes.next() {
        let rest: String = graphemes.collect();
        let capitalized = cm.uppercase_to_string(first, &locale);
        Ok(format!("{}{}", capitalized, rest))
    } else {
        Ok(text)
    }
}

fn parse_langid(lang: &str) -> icu_locale_core::LanguageIdentifier {
    match lang {
        "tr" => langid!("tr"),
        "az" => langid!("az"), // Azerbaijani has same i/I as Turkish
        // ... other special cases ...
        _ => langid!("und"), // "und" = undetermined, uses root locale
    }
}
```

### Anti-Patterns to Avoid
- **Using str::to_uppercase() directly:** NOT locale-aware. Turkish "istanbul" should become "ISTANBUL" not "ISTANBUL".
- **Using chars().next() for first letter:** Mishandles combining characters. "e\u{0301}" (e with combining acute) is one grapheme but two chars.
- **Caching CaseMapper per-call:** `CaseMapper::new()` is cheap (compiled data). Cache at registry level if profiling shows need.
- **Trait objects for transforms:** CONTEXT.md explicitly requires static dispatch via enum.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Locale-sensitive uppercase | `str.to_uppercase()` | `CaseMapper::uppercase_to_string()` | Turkish i->I is wrong without locale |
| First grapheme extraction | `.chars().next()` | `.graphemes(true).next()` | Combining characters (accents) need UAX#29 handling |
| Language ID parsing | string matching | `langid!()` macro | Compile-time validation, proper subtag handling |

**Key insight:** Case mapping is locale-dependent. Turkish has four distinct I letters (i, I, i, I) and standard Rust `to_uppercase()` does not handle this. Greek uppercasing removes accents. These edge cases make ICU4X essential for correctness.

## Common Pitfalls

### Pitfall 1: Turkish i/I Case Mapping
**What goes wrong:** "istanbul" uppercased to "ISTANBUL" instead of "ISTANBUL" (with dotted capital I).
**Why it happens:** Rust stdlib `to_uppercase()` is not locale-aware.
**How to avoid:** Use `CaseMapper::uppercase_to_string(text, &langid!("tr"))` for Turkish.
**Warning signs:** Tests pass in English, fail for Turkish strings.

### Pitfall 2: Combining Characters in @cap
**What goes wrong:** "e\u{0301}xample" (e + combining acute) capitalizes only the "e", leaving combining acute orphaned.
**Why it happens:** `.chars().next()` returns single char, not grapheme cluster.
**How to avoid:** Use `text.graphemes(true).next()` from unicode-segmentation.
**Warning signs:** Accented text displays incorrectly after capitalization.

### Pitfall 3: Transform Context vs Selector Confusion
**What goes wrong:** Transform context `:acc` in `@der:acc` confused with selector `:one` in `{card:one}`.
**Why it happens:** Both use `:` syntax.
**How to avoid:** Parser already distinguishes these. Transform context is in `Transform.context`, selectors are in `Interpolation.selectors`. Don't mix them in execution.
**Warning signs:** German article transform gets "one" instead of "acc".

### Pitfall 4: Transform Order (Right-to-Left)
**What goes wrong:** `{@cap @a card}` produces "A Card" instead of "A card" (cap applied first, then a).
**Why it happens:** Processing transforms left-to-right instead of right-to-left.
**How to avoid:** Iterate `transforms.iter().rev()` to process innermost (rightmost) first.
**Warning signs:** Capitalization appears in wrong position relative to articles.

### Pitfall 5: Empty String Edge Case
**What goes wrong:** @cap on "" panics or produces garbage.
**Why it happens:** Not checking for empty input before grapheme iteration.
**How to avoid:** Early return `""` if input is empty. Document this as specified behavior in CONTEXT.md.
**Warning signs:** Tests with empty strings fail.

## Code Examples

Verified patterns from official sources:

### ICU4X Locale-Sensitive Uppercase
```rust
// Source: https://docs.rs/icu_casemap/2.1.1/icu_casemap/
use icu_casemap::CaseMapper;
use icu_locale_core::langid;

let cm = CaseMapper::new();

// English: normal uppercase
assert_eq!(
    cm.uppercase_to_string("hello", &langid!("en")),
    "HELLO"
);

// Turkish: dotted I preserved
assert_eq!(
    cm.uppercase_to_string("istanbul", &langid!("tr")),
    "ISTANBUL"  // Note: I with dot above
);
```

### Unicode Segmentation for Graphemes
```rust
// Source: https://docs.rs/unicode-segmentation/1.12.0/unicode_segmentation/
use unicode_segmentation::UnicodeSegmentation;

let s = "a\u{0308}bc"; // a + combining diaeresis + bc = "abc"

// Extended grapheme clusters (recommended)
let graphemes: Vec<&str> = s.graphemes(true).collect();
assert_eq!(graphemes, vec!["a\u{0308}", "b", "c"]); // 3 graphemes, not 4 chars
```

### Full @cap Implementation
```rust
// Source: Combining ICU4X + unicode-segmentation for correct @cap
use icu_casemap::CaseMapper;
use icu_locale_core::LanguageIdentifier;
use unicode_segmentation::UnicodeSegmentation;

pub fn cap_transform(text: &str, locale: &LanguageIdentifier) -> String {
    if text.is_empty() {
        return String::new();
    }

    let cm = CaseMapper::new();
    let mut graphemes = text.graphemes(true);

    match graphemes.next() {
        Some(first) => {
            let rest: String = graphemes.collect();
            let capitalized = cm.uppercase_to_string(first, locale);
            format!("{}{}", capitalized, rest)
        }
        None => String::new(),
    }
}

// Usage
let result = cap_transform("istanbul", &langid!("tr"));
// Result: "Istanbul" with dotted I
```

### @upper and @lower Implementation
```rust
// Source: ICU4X CaseMapper API
use icu_casemap::CaseMapper;
use icu_locale_core::LanguageIdentifier;

pub fn upper_transform(text: &str, locale: &LanguageIdentifier) -> String {
    let cm = CaseMapper::new();
    cm.uppercase_to_string(text, locale)
}

pub fn lower_transform(text: &str, locale: &LanguageIdentifier) -> String {
    let cm = CaseMapper::new();
    cm.lowercase_to_string(text, locale)
}
```

### Transform Registry Initialization
```rust
// Source: Designed based on CONTEXT.md decisions
pub fn init_transform_registry() -> TransformRegistry {
    let mut registry = TransformRegistry::new();

    // Universal transforms are always registered
    // No explicit registration needed - TransformKind::get() handles them

    // Language-specific transforms will be added in Phases 6-9
    // registry.register_language("en", vec![("a", TransformKind::A)]);
    // registry.register_language("de", vec![("der", TransformKind::Der)]);

    registry
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| str::to_uppercase() | icu_casemap::CaseMapper | ICU4X 1.3 (2023) | Proper Turkish, Greek case mapping |
| Custom grapheme logic | unicode-segmentation crate | Stable since ~2019 | UAX#29 compliant, well-tested |
| Function pointers for transforms | Static dispatch via enum | Design decision | No dynamic dispatch overhead, exhaustive match checking |

**Deprecated/outdated:**
- `unic-segment`: Older Unicode crate, `unicode-segmentation` is more actively maintained.
- Custom locale parsing: Use `icu_locale_core::langid!()` macro for compile-time validation.

## Open Questions

Things that couldn't be fully resolved:

1. **CaseMapper Caching Strategy**
   - What we know: CONTEXT.md marks this as Claude's discretion
   - What's unclear: Should we cache CaseMapper per-language in TransformRegistry?
   - Recommendation: Start without caching. `CaseMapper::new()` uses compiled data and is documented as cheap. Profile first, cache if needed. If caching, use `HashMap<String, CaseMapper>` in TransformRegistry.

2. **Error Type for Unknown Transform**
   - What we know: EvalError exists, doesn't have UnknownTransform variant yet
   - What's unclear: Should this be a separate variant or reuse existing?
   - Recommendation: Add `EvalError::UnknownTransform { name: String }` variant for clarity in error messages.

3. **Transform Alias Handling Location**
   - What we know: CONTEXT.md says aliases (@an -> @a) "map to same enum variant in parser"
   - What's unclear: Should aliases be normalized in parser or in TransformRegistry::get()?
   - Recommendation: Handle in parser for consistency - `@an` parses to `Transform { name: "a", ... }`. Simpler than maintaining alias map in registry.

## Sources

### Primary (HIGH confidence)
- [icu_casemap 2.1.1 docs.rs](https://docs.rs/icu_casemap/2.1.1/icu_casemap/) - CaseMapper API, locale-sensitive case operations
- [unicode-segmentation 1.12.0 docs.rs](https://docs.rs/unicode-segmentation/1.12.0/unicode_segmentation/) - UnicodeSegmentation trait, graphemes() method
- docs/DESIGN.md - RLF transform specification, right-to-left execution order
- docs/APPENDIX_STDLIB.md - Universal transform definitions (@cap, @upper, @lower)
- docs/APPENDIX_RUST_INTEGRATION.md - Transform function signatures
- .planning/phases/03-universal-transforms-and-icu4x/03-CONTEXT.md - User decisions constraining implementation

### Secondary (MEDIUM confidence)
- [Unicode Blog - ICU4X 1.3 Release](https://blog.unicode.org/2023/10/icu4x-13-now-with-built-in-data-case.html) - Case mapping feature announcement
- [GitHub unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) - Implementation details, examples

### Tertiary (LOW confidence)
- None - all findings verified with official documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - ICU4X is official Unicode implementation, unicode-segmentation is de facto standard
- Architecture: HIGH - Patterns derived from CONTEXT.md decisions and design docs
- Pitfalls: HIGH - Turkish case mapping and grapheme handling are well-documented issues

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - ICU4X 2.x is stable, unicode-segmentation 1.x is stable)
