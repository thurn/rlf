# Phase 6: English and Germanic Transforms - Research

**Researched:** 2026-02-04
**Domain:** Language-specific article transforms for English, German, and Dutch
**Confidence:** HIGH

## Summary

This phase implements metadata-driven article transforms for English, German, and Dutch. The core infrastructure already exists from Phase 3: `TransformKind` enum with static dispatch, `TransformRegistry` for lookup, and the evaluator's `apply_transforms` function with context support. Phase 6 extends `TransformKind` with new variants for language-specific transforms that read phrase metadata tags to determine article selection.

The approach follows the established patterns: transforms check for required tags on the input `Value` (specifically `Value::Phrase`), select the appropriate article form based on tags and optional context, and return the article prepended to the phrase text. Missing tags produce `EvalError::MissingTag`. Transform aliases (e.g., `@an` maps to `@a`, `@die` maps to `@der`) are handled in `TransformRegistry::get()`.

Key technical decisions:
1. English transforms (`@a`/`@an`, `@the`) read `:a`/`:an` tags or produce static "the"
2. German transforms (`@der`/`@die`/`@das`, `@ein`/`@eine`) use gender tags (`:masc`, `:fem`, `:neut`) and case context (`:nom`, `:acc`, `:dat`, `:gen`)
3. Dutch transforms (`@de`/`@het`, `@een`) read `:de`/`:het` tags for definite articles
4. All transforms receive the original `Value` (not just string) to access phrase tags

**Primary recommendation:** Extend `TransformKind` enum with `EnglishA`, `EnglishThe`, `GermanDer`, `GermanEin`, `DutchDe`, `DutchEen` variants. Implement tag checking with `Value::has_tag()` and context parsing for German case system. Register transforms per-language with alias resolution in `TransformRegistry::get()`.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rlf (existing) | - | Transform infrastructure | TransformKind, TransformRegistry, apply_transforms already implemented |
| thiserror | 2.0 | Error types (existing) | EvalError::MissingTag variant already exists |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | - | No additional dependencies | Language transforms are pure logic using existing infrastructure |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static enum variants | HashMap<String, fn> | Violates CONTEXT.md decision - static dispatch only |
| Tag-based selection | Phonetic analysis | DESIGN.md explicitly forbids phonetic guessing |

**Installation:**
```bash
# No additional dependencies required
# All infrastructure exists from Phase 3
```

## Architecture Patterns

### Recommended Project Structure
```
crates/rlf/src/
  interpreter/
    transforms.rs      # Extended TransformKind enum, language-specific implementations
    evaluator.rs       # No changes needed - context already supported
    mod.rs             # Exports (no changes)
crates/rlf/tests/
    interpreter_transforms.rs  # Extended with language-specific tests
```

### Pattern 1: Extended TransformKind Enum
**What:** Add new variants to `TransformKind` for each language-specific transform.
**When to use:** For all new transforms in this phase.
**Example:**
```rust
// Source: transforms.rs - existing enum extended
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    // Universal transforms (existing)
    Cap,
    Upper,
    Lower,

    // English transforms (Phase 6)
    EnglishA,     // @a/@an - indefinite article from :a/:an tags
    EnglishThe,   // @the - definite article "the"

    // German transforms (Phase 6)
    GermanDer,    // @der/@die/@das - definite article with case context
    GermanEin,    // @ein/@eine - indefinite article with case context

    // Dutch transforms (Phase 6)
    DutchDe,      // @de/@het - definite article from :de/:het tags
    DutchEen,     // @een - indefinite article "een"
}
```

### Pattern 2: Metadata-Driven Transform Implementation
**What:** Transforms read tags from `Value::Phrase` to determine output.
**When to use:** All transforms that depend on phrase metadata.
**Example:**
```rust
// Source: DESIGN.md - @a transform reads :a/:an tags
fn english_a_transform(value: &Value, _context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    // Check for required tags
    if value.has_tag("a") {
        return Ok(format!("a {}", text));
    }
    if value.has_tag("an") {
        return Ok(format!("an {}", text));
    }

    // Missing tag - produce error per DESIGN.md
    Err(EvalError::MissingTag {
        transform: "a".to_string(),
        expected: vec!["a".to_string(), "an".to_string()],
        phrase: text,
    })
}
```

### Pattern 3: Transform Context for Case Selection
**What:** German transforms use context parameter for grammatical case.
**When to use:** `@der:acc`, `@ein:dat`, etc.
**Example:**
```rust
// Source: DESIGN.md - transform context syntax @transform:context
fn german_der_transform(
    value: &Value,
    context: Option<&Value>,
    _lang: &str,
) -> Result<String, EvalError> {
    let text = value.to_string();

    // Determine gender from tags
    let gender = if value.has_tag("masc") {
        Gender::Masculine
    } else if value.has_tag("fem") {
        Gender::Feminine
    } else if value.has_tag("neut") {
        Gender::Neuter
    } else {
        return Err(EvalError::MissingTag {
            transform: "der".to_string(),
            expected: vec!["masc".to_string(), "fem".to_string(), "neut".to_string()],
            phrase: text,
        });
    };

    // Determine case from context (default to nominative)
    let case = parse_german_case(context);

    // Select article form
    let article = german_definite_article(gender, case);
    Ok(format!("{} {}", article, text))
}

fn parse_german_case(context: Option<&Value>) -> GermanCase {
    match context {
        Some(Value::String(s)) => match s.as_str() {
            "nom" => GermanCase::Nominative,
            "acc" => GermanCase::Accusative,
            "dat" => GermanCase::Dative,
            "gen" => GermanCase::Genitive,
            _ => GermanCase::Nominative, // Default
        },
        _ => GermanCase::Nominative,
    }
}
```

### Pattern 4: Transform Alias Resolution in Registry
**What:** Map alias names to canonical transform names in `TransformRegistry::get()`.
**When to use:** All aliased transforms (@an -> @a, @die -> @der, etc.).
**Example:**
```rust
// Source: transforms.rs - TransformRegistry::get() extended
impl TransformRegistry {
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformKind> {
        // Resolve aliases first
        let canonical = match name {
            // English aliases
            "an" => "a",
            // German aliases
            "die" | "das" => "der",
            "eine" => "ein",
            // Dutch aliases
            "het" => "de",
            // No alias
            other => other,
        };

        // Universal transforms (always available)
        match canonical {
            "cap" => return Some(TransformKind::Cap),
            "upper" => return Some(TransformKind::Upper),
            "lower" => return Some(TransformKind::Lower),
            _ => {}
        }

        // Language-specific transforms
        match (lang, canonical) {
            // English
            ("en", "a") => Some(TransformKind::EnglishA),
            ("en", "the") => Some(TransformKind::EnglishThe),
            // German
            ("de", "der") => Some(TransformKind::GermanDer),
            ("de", "ein") => Some(TransformKind::GermanEin),
            // Dutch
            ("nl", "de") => Some(TransformKind::DutchDe),
            ("nl", "een") => Some(TransformKind::DutchEen),
            _ => None,
        }
    }
}
```

### Pattern 5: Context Resolution in Evaluator
**What:** Resolve transform context selector to a value before passing to transform.
**When to use:** When evaluating transforms with context (e.g., `@der:acc`).
**Example:**
```rust
// Source: evaluator.rs - apply_transforms needs context resolution
fn apply_transforms(
    initial_value: &str,
    transforms: &[Transform],
    transform_registry: &TransformRegistry,
    ctx: &EvalContext<'_>,  // Needed for parameter-based context
    lang: &str,
) -> Result<String, EvalError> {
    // ... existing code ...

    for transform in transforms.iter().rev() {
        // Resolve context if present
        let context_value = if let Some(ctx_selector) = &transform.context {
            match ctx_selector {
                Selector::Identifier(name) => {
                    // Try parameter lookup first
                    if let Some(param) = ctx.get_param(name) {
                        Some(param.clone())
                    } else {
                        // Use as literal string
                        Some(Value::String(name.clone()))
                    }
                }
            }
        } else {
            None
        };

        let result = transform_kind.execute(&current, context_value.as_ref(), lang)?;
        // ...
    }
}
```

### Anti-Patterns to Avoid
- **Phonetic analysis for @a/@an:** DESIGN.md explicitly forbids this. Tags are authoritative.
- **Hardcoded language checks in execute():** Use `TransformRegistry::get()` for language dispatch.
- **Ignoring context parameter:** German transforms must honor case context, defaulting to nominative.
- **Treating aliases as separate variants:** @an and @a should resolve to the same `TransformKind::EnglishA`.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Article selection | Custom if-else chains | Tag-based dispatch | Consistent pattern, testable, extensible |
| Context parsing | String parsing everywhere | Dedicated parse function | Reusable, handles defaults |
| Alias resolution | Duplicate enum variants | Map in get() | Single source of truth |

**Key insight:** The transform system is already designed for this use case. The Value type has `has_tag()`, the Transform AST has `context`, and `EvalError::MissingTag` exists. Phase 6 connects these pieces.

## Common Pitfalls

### Pitfall 1: Forgetting to Pass Value (Not String) to Transforms
**What goes wrong:** Transform receives string, can't access tags, always errors.
**Why it happens:** Converting to string too early in apply_transforms.
**How to avoid:** Pass `Value` to transform functions. Convert to string only inside transform after tag checking.
**Warning signs:** All tag-dependent transforms fail with MissingTag.

### Pitfall 2: German Case Context Defaults
**What goes wrong:** `@der karte` without context fails or produces wrong article.
**Why it happens:** Not handling missing context.
**How to avoid:** Default to nominative case when context is None. Document this behavior.
**Warning signs:** Simple German examples without context fail.

### Pitfall 3: Dutch Gender Tag Names
**What goes wrong:** Dutch uses `:de`/`:het` tags (matching article names), not `:masc`/`:fem`.
**Why it happens:** Assuming Germanic languages share tag conventions.
**How to avoid:** Dutch specifically uses `:de` for common gender, `:het` for neuter. Check APPENDIX_STDLIB.md.
**Warning signs:** Dutch phrases with `:masc` tag produce MissingTag errors.

### Pitfall 4: Transform Order with Articles
**What goes wrong:** `{@cap @a card}` produces "a Card" instead of "A card".
**Why it happens:** Capitalization applied to article, not just first letter of combined result.
**How to avoid:** Transform execution is right-to-left: @a runs first producing "a card", then @cap capitalizes to "A card".
**Warning signs:** Test outputs have wrong capitalization.

### Pitfall 5: Alias and Language Mismatch
**What goes wrong:** `@de` in English context doesn't resolve (it's Dutch, not English).
**Why it happens:** Aliases registered globally instead of per-language.
**How to avoid:** Language check happens in `TransformRegistry::get()` before returning transform.
**Warning signs:** Transforms work in wrong language.

## Code Examples

Verified patterns from design documents and existing code:

### English @a/@an Transform
```rust
// Source: DESIGN.md + APPENDIX_STDLIB.md
fn english_a_transform(value: &Value, _context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("a") {
        Ok(format!("a {}", text))
    } else if value.has_tag("an") {
        Ok(format!("an {}", text))
    } else {
        Err(EvalError::MissingTag {
            transform: "a".to_string(),
            expected: vec!["a".to_string(), "an".to_string()],
            phrase: text,
        })
    }
}
```

### English @the Transform
```rust
// Source: APPENDIX_STDLIB.md - @the always produces "the"
fn english_the_transform(value: &Value, _context: Option<&Value>) -> Result<String, EvalError> {
    Ok(format!("the {}", value.to_string()))
}
```

### German Definite Article Tables
```rust
// Source: German grammar - definite article declension
enum Gender { Masculine, Feminine, Neuter }
enum GermanCase { Nominative, Accusative, Dative, Genitive }

fn german_definite_article(gender: Gender, case: GermanCase) -> &'static str {
    match (gender, case) {
        // Masculine
        (Gender::Masculine, GermanCase::Nominative) => "der",
        (Gender::Masculine, GermanCase::Accusative) => "den",
        (Gender::Masculine, GermanCase::Dative) => "dem",
        (Gender::Masculine, GermanCase::Genitive) => "des",
        // Feminine (same for nom/acc, different for dat/gen)
        (Gender::Feminine, GermanCase::Nominative) => "die",
        (Gender::Feminine, GermanCase::Accusative) => "die",
        (Gender::Feminine, GermanCase::Dative) => "der",
        (Gender::Feminine, GermanCase::Genitive) => "der",
        // Neuter
        (Gender::Neuter, GermanCase::Nominative) => "das",
        (Gender::Neuter, GermanCase::Accusative) => "das",
        (Gender::Neuter, GermanCase::Dative) => "dem",
        (Gender::Neuter, GermanCase::Genitive) => "des",
    }
}

fn german_indefinite_article(gender: Gender, case: GermanCase) -> &'static str {
    match (gender, case) {
        // Masculine
        (Gender::Masculine, GermanCase::Nominative) => "ein",
        (Gender::Masculine, GermanCase::Accusative) => "einen",
        (Gender::Masculine, GermanCase::Dative) => "einem",
        (Gender::Masculine, GermanCase::Genitive) => "eines",
        // Feminine
        (Gender::Feminine, GermanCase::Nominative) => "eine",
        (Gender::Feminine, GermanCase::Accusative) => "eine",
        (Gender::Feminine, GermanCase::Dative) => "einer",
        (Gender::Feminine, GermanCase::Genitive) => "einer",
        // Neuter
        (Gender::Neuter, GermanCase::Nominative) => "ein",
        (Gender::Neuter, GermanCase::Accusative) => "ein",
        (Gender::Neuter, GermanCase::Dative) => "einem",
        (Gender::Neuter, GermanCase::Genitive) => "eines",
    }
}
```

### Dutch Definite Article Transform
```rust
// Source: APPENDIX_STDLIB.md - Dutch :de/:het tags
fn dutch_de_transform(value: &Value, _context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("de") {
        Ok(format!("de {}", text))
    } else if value.has_tag("het") {
        Ok(format!("het {}", text))
    } else {
        Err(EvalError::MissingTag {
            transform: "de".to_string(),
            expected: vec!["de".to_string(), "het".to_string()],
            phrase: text,
        })
    }
}

fn dutch_een_transform(value: &Value, _context: Option<&Value>) -> Result<String, EvalError> {
    // Dutch indefinite article is always "een" regardless of gender
    Ok(format!("een {}", value.to_string()))
}
```

### TransformKind::execute() Extension
```rust
// Source: transforms.rs - extended execute method
impl TransformKind {
    pub fn execute(
        &self,
        value: &Value,
        context: Option<&Value>,
        lang: &str,
    ) -> Result<String, EvalError> {
        match self {
            // Universal transforms (existing)
            TransformKind::Cap => cap_transform(&value.to_string(), &parse_langid(lang)),
            TransformKind::Upper => upper_transform(&value.to_string(), &parse_langid(lang)),
            TransformKind::Lower => lower_transform(&value.to_string(), &parse_langid(lang)),

            // English transforms
            TransformKind::EnglishA => english_a_transform(value, context),
            TransformKind::EnglishThe => english_the_transform(value, context),

            // German transforms
            TransformKind::GermanDer => german_der_transform(value, context),
            TransformKind::GermanEin => german_ein_transform(value, context),

            // Dutch transforms
            TransformKind::DutchDe => dutch_de_transform(value, context),
            TransformKind::DutchEen => dutch_een_transform(value, context),
        }
    }
}
```

## German Case System Reference

German has four grammatical cases that affect article form:

| Case | Question | Usage Example |
|------|----------|---------------|
| Nominative (nom) | Wer/Was? (Who/What?) | Subject of sentence |
| Accusative (acc) | Wen/Was? (Whom/What?) | Direct object |
| Dative (dat) | Wem? (To whom?) | Indirect object |
| Genitive (gen) | Wessen? (Whose?) | Possession |

**Definite Articles:**

| Case | Masculine | Feminine | Neuter | Plural |
|------|-----------|----------|--------|--------|
| Nominative | der | die | das | die |
| Accusative | den | die | das | die |
| Dative | dem | der | dem | den |
| Genitive | des | der | des | der |

**Indefinite Articles:**

| Case | Masculine | Feminine | Neuter |
|------|-----------|----------|--------|
| Nominative | ein | eine | ein |
| Accusative | einen | eine | ein |
| Dative | einem | einer | einem |
| Genitive | eines | einer | eines |

Note: Phase 6 focuses on singular forms. Plural handling can be added if needed.

## Dutch Gender System Reference

Dutch has two grammatical genders for articles:

| Gender | Definite Article | Example |
|--------|------------------|---------|
| Common (de-words) | de | de kaart (the card) |
| Neuter (het-words) | het | het karakter (the character) |

**Tags:**
- `:de` - Common gender (historically masculine or feminine)
- `:het` - Neuter gender

**Indefinite article:** Always "een" regardless of gender.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phonetic heuristics for a/an | Tag-based selection | RLF design decision | No incorrect articles for edge cases |
| Separate enum per language | Unified TransformKind | Phase 3 design | Single dispatch point, exhaustive matching |
| Manual context parsing | Context parameter in Transform AST | Phase 1 | Consistent syntax across languages |

**Deprecated/outdated:**
- Phonetic analysis: Never implemented, explicitly rejected in DESIGN.md
- Function pointer dispatch: Rejected in favor of static enum dispatch

## Open Questions

Things that couldn't be fully resolved:

1. **Plural Articles for German**
   - What we know: German plural has "die" for all genders (definite), no indefinite plural
   - What's unclear: Should `@der:nom:other` produce "die"? Or is that out of scope?
   - Recommendation: Focus on singular for Phase 6. Plural can be added by checking selector context or with separate transform.

2. **Context as Parameter Reference**
   - What we know: `@der:acc` uses literal "acc". But `@der:n` where n is a parameter could work.
   - What's unclear: How should parameter-based context resolve (to plural category? to literal string?)?
   - Recommendation: Start with literal context strings. If parameter context needed, resolve as string value and match against known cases.

3. **Error Message Quality**
   - What we know: `EvalError::MissingTag` exists with transform name, expected tags, phrase text
   - What's unclear: Should we include language in error? ("@der requires :masc/:fem/:neut for German")
   - Recommendation: Keep current error format for consistency. Language context is clear from usage.

## Sources

### Primary (HIGH confidence)
- docs/DESIGN.md - Transform specification, metadata tag usage, context syntax
- docs/APPENDIX_STDLIB.md - English/German/Dutch transform definitions and examples
- docs/APPENDIX_RUST_INTEGRATION.md - Transform function signatures
- crates/rlf/src/interpreter/transforms.rs - Existing TransformKind implementation
- crates/rlf/src/interpreter/evaluator.rs - apply_transforms implementation

### Secondary (MEDIUM confidence)
- .planning/phases/03-universal-transforms-and-icu4x/03-RESEARCH.md - Transform architecture patterns
- Standard German grammar references for article declension tables

### Tertiary (LOW confidence)
- None - all findings verified with project documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, extending existing infrastructure
- Architecture: HIGH - Patterns established in Phase 3, documented in CONTEXT.md
- Language rules: HIGH - German/Dutch article tables are standard grammar, English is straightforward

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - stable domain, no external dependencies changing)
