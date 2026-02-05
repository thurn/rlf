# Phase 7: Romance Language Transforms - Research

**Researched:** 2026-02-04
**Domain:** Article and contraction transforms for Spanish, French, Portuguese, Italian
**Confidence:** HIGH

## Summary

This phase implements article and contraction transforms for four Romance languages. The core infrastructure from Phase 6 (TransformKind enum, TransformRegistry, tag-based article selection) directly applies. Phase 7 extends TransformKind with new variants for each language's transforms, following the established patterns.

The languages divide into two complexity tiers:
1. **Spanish + Portuguese (simpler):** Gender-based definite/indefinite articles with plural context. Portuguese adds two contraction transforms (@de, @em).
2. **French + Italian (complex):** Same base functionality plus elision rules requiring :vowel tag (and Italian's :s_imp for s-impura words).

Per CONTEXT.md decisions: all Romance languages use :masc/:fem tags (no :neut), context selectors handle plural forms (:one/:other), and missing tags always produce errors. The pattern from German's case context (@der:acc) extends to plural context (@el:other -> los/las).

**Primary recommendation:** Split into two plans following CONTEXT.md: Plan 1 (Spanish + Portuguese) focuses on gender/number article selection with Portuguese contractions. Plan 2 (French + Italian) adds elision complexity and Italian sound rules.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rlf (existing) | - | Transform infrastructure | TransformKind, TransformRegistry, context resolution all exist |
| thiserror | 2.0 | Error types (existing) | EvalError::MissingTag for tag validation |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | - | No additional dependencies | Pure lookup tables and string formatting |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Lookup tables | Match expressions | Match is explicit, easier to verify against spec |
| Context for plural | New transform variant | Context pattern established in Phase 6, reuse it |

**Installation:**
```bash
# No additional dependencies required
# All infrastructure exists from Phase 6
```

## Architecture Patterns

### Recommended Project Structure
```
crates/rlf/src/
  interpreter/
    transforms.rs      # Extended TransformKind enum with Romance variants
    evaluator.rs       # No changes needed - context already supported
crates/rlf/tests/
    interpreter_transforms.rs  # Extended with Romance language tests
```

### Pattern 1: Romance Gender-Article Lookup
**What:** Two-gender (masc/fem) article selection, simpler than German's three-gender system.
**When to use:** Spanish, French, Portuguese, Italian definite and indefinite articles.
**Example:**
```rust
// Source: APPENDIX_STDLIB.md - Spanish @el transform
enum RomanceGender {
    Masculine,
    Feminine,
}

enum PluralCategory {
    One,
    Other,
}

fn spanish_definite_article(gender: RomanceGender, plural: PluralCategory) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, PluralCategory::One) => "el",
        (RomanceGender::Masculine, PluralCategory::Other) => "los",
        (RomanceGender::Feminine, PluralCategory::One) => "la",
        (RomanceGender::Feminine, PluralCategory::Other) => "las",
    }
}
```

### Pattern 2: Plural Context Resolution
**What:** Use context selector (:one/:other) for singular/plural article forms.
**When to use:** All Romance article transforms supporting `@el:other` syntax.
**Example:**
```rust
// Source: APPENDIX_STDLIB.md - context selector pattern
fn parse_romance_plural(context: Option<&Value>) -> PluralCategory {
    match context {
        Some(Value::String(s)) => match s.as_str() {
            "other" => PluralCategory::Other,
            _ => PluralCategory::One, // Default to singular
        },
        Some(Value::Number(n)) => {
            // If numeric context, use plural rules
            if *n == 1 { PluralCategory::One } else { PluralCategory::Other }
        },
        _ => PluralCategory::One, // Default
    }
}
```

### Pattern 3: Elision with Vowel Tag
**What:** French/Italian articles change form before vowels (l'ami, l'amico).
**When to use:** French @le transform, Italian @il transform.
**Example:**
```rust
// Source: APPENDIX_STDLIB.md - French elision
fn french_definite_article(
    gender: RomanceGender,
    has_vowel: bool,
    plural: PluralCategory,
) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision before vowel (singular only)
        (_, true, PluralCategory::One) => "l'",
        // Masculine singular
        (RomanceGender::Masculine, false, PluralCategory::One) => "le",
        // Feminine singular
        (RomanceGender::Feminine, false, PluralCategory::One) => "la",
        // Plural (same for both genders)
        (_, _, PluralCategory::Other) => "les",
    }
}
```

### Pattern 4: Contraction Transforms
**What:** Preposition + article combined forms (French "du", Portuguese "no").
**When to use:** French @de/@au, Portuguese @de/@em.
**Example:**
```rust
// Source: APPENDIX_STDLIB.md - French @de contraction
fn french_de_contraction(
    gender: RomanceGender,
    has_vowel: bool,
    plural: PluralCategory,
) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision: de + l' -> de l' (no contraction)
        (_, true, PluralCategory::One) => "de l'",
        // Masculine singular: de + le -> du
        (RomanceGender::Masculine, false, PluralCategory::One) => "du",
        // Feminine singular: de + la -> de la (no contraction)
        (RomanceGender::Feminine, false, PluralCategory::One) => "de la",
        // Plural: de + les -> des
        (_, _, PluralCategory::Other) => "des",
    }
}
```

### Pattern 5: Italian Sound-Based Article Selection
**What:** Italian articles vary based on initial sound (:s_imp for s+consonant/z/gn/ps/x).
**When to use:** Italian @il/@un transforms.
**Example:**
```rust
// Source: APPENDIX_STDLIB.md - Italian sound rules
enum ItalianSound {
    Normal,     // Standard consonant (il/un)
    Vowel,      // Starts with vowel (l'/un)
    SImpura,    // s+cons, z, gn, ps, x (lo/uno)
}

fn italian_definite_article(
    gender: RomanceGender,
    sound: ItalianSound,
    plural: PluralCategory,
) -> &'static str {
    match (gender, sound, plural) {
        // Masculine singular variants
        (RomanceGender::Masculine, ItalianSound::Normal, PluralCategory::One) => "il",
        (RomanceGender::Masculine, ItalianSound::Vowel, PluralCategory::One) => "l'",
        (RomanceGender::Masculine, ItalianSound::SImpura, PluralCategory::One) => "lo",
        // Masculine plural
        (RomanceGender::Masculine, ItalianSound::Normal, PluralCategory::Other) => "i",
        (RomanceGender::Masculine, ItalianSound::Vowel, PluralCategory::Other) => "gli",
        (RomanceGender::Masculine, ItalianSound::SImpura, PluralCategory::Other) => "gli",
        // Feminine singular
        (RomanceGender::Feminine, ItalianSound::Vowel, PluralCategory::One) => "l'",
        (RomanceGender::Feminine, _, PluralCategory::One) => "la",
        // Feminine plural
        (RomanceGender::Feminine, _, PluralCategory::Other) => "le",
    }
}
```

### Anti-Patterns to Avoid
- **Automatic vowel detection:** CONTEXT.md explicitly forbids this - use :vowel tag
- **Assuming three genders:** Romance languages only have masc/fem (not neut)
- **Creating @do/@no aliases:** CONTEXT.md says Portuguese uses distinct transform names only
- **Hardcoding Spanish contractions:** APPENDIX_STDLIB shows `"de {@el x}"` pattern, not dedicated transforms

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Plural context parsing | Custom string parsing | Reuse parse_german_case pattern | Consistent context resolution |
| Gender enum | Per-language enum | Shared RomanceGender | Same semantics across Romance languages |
| Transform aliases | Duplicate enum variants | Map in registry.get() | Phase 6 pattern established |
| Apostrophe handling | Unicode detection | ASCII `'` per CONTEXT.md | Decision already made |

**Key insight:** The Phase 6 infrastructure handles 90% of the complexity. Romance transforms are essentially lookup tables with the same tag-checking and context-resolution patterns.

## Common Pitfalls

### Pitfall 1: Forgetting Elision Apostrophe Formatting
**What goes wrong:** Output is "l ami" instead of "l'ami".
**Why it happens:** Article string doesn't include the apostrophe.
**How to avoid:** Elided articles return "l'" (with apostrophe), not "l" + space.
**Warning signs:** Tests fail with missing apostrophe.

### Pitfall 2: Italian Sound Tag Ambiguity
**What goes wrong:** Transform doesn't know if word is :s_imp or :vowel or neither.
**Why it happens:** Multiple sound states but no default.
**How to avoid:** Per CONTEXT.md - require :s_imp or :vowel tags when applicable, error if ambiguous (neither :masc nor :fem tag).
**Warning signs:** Italian words without sound tags silently get wrong article.

### Pitfall 3: Portuguese vs French @de Collision
**What goes wrong:** @de in Portuguese uses different contraction table than French.
**Why it happens:** Same transform name, different languages.
**How to avoid:** TransformRegistry::get() checks language parameter. Portuguese @de is distinct from French @de.
**Warning signs:** Tests using wrong contraction in cross-language scenarios.

### Pitfall 4: Context Priority Confusion
**What goes wrong:** `@el:other word:one` produces unexpected result.
**Why it happens:** Context applies to transform, selector applies to phrase.
**How to avoid:** Document clearly: `:other` after `@el` is transform context (plural), `:one` after phrase is variant selector.
**Warning signs:** Mixed context/selector tests fail.

### Pitfall 5: French Indefinite Plural
**What goes wrong:** Attempt to handle `@un:other` which doesn't exist in French.
**Why it happens:** Assuming indefinite articles have plural forms like definite.
**How to avoid:** Per APPENDIX_STDLIB - French @un/@une has no plural context support.
**Warning signs:** Tests attempt @un:other which should remain singular.

### Pitfall 6: Spanish "el agua" Exception
**What goes wrong:** Feminine noun with :fem tag produces "la agua" (incorrect).
**Why it happens:** Spanish uses "el" before stressed-a feminine nouns.
**How to avoid:** This is out of scope per APPENDIX_STDLIB - users must tag accordingly or handle in phrase definition.
**Warning signs:** Edge case tests with stressed-a words.

## Code Examples

Verified patterns from APPENDIX_STDLIB and existing code:

### Spanish @el Transform (Complete)
```rust
// Source: APPENDIX_STDLIB.md + Phase 6 pattern
fn spanish_el_transform(
    value: &Value,
    context: Option<&Value>,
) -> Result<String, EvalError> {
    let text = value.to_string();

    // Determine gender
    let gender = if value.has_tag("masc") {
        RomanceGender::Masculine
    } else if value.has_tag("fem") {
        RomanceGender::Feminine
    } else {
        return Err(EvalError::MissingTag {
            transform: "el".to_string(),
            expected: vec!["masc".to_string(), "fem".to_string()],
            phrase: text,
        });
    };

    // Determine plural from context
    let plural = parse_romance_plural(context);

    // Select article
    let article = spanish_definite_article(gender, plural);
    Ok(format!("{} {}", article, text))
}

fn spanish_definite_article(gender: RomanceGender, plural: PluralCategory) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, PluralCategory::One) => "el",
        (RomanceGender::Masculine, PluralCategory::Other) => "los",
        (RomanceGender::Feminine, PluralCategory::One) => "la",
        (RomanceGender::Feminine, PluralCategory::Other) => "las",
    }
}
```

### Portuguese @de Contraction Transform
```rust
// Source: APPENDIX_STDLIB.md - Portuguese contractions
fn portuguese_de_transform(
    value: &Value,
    context: Option<&Value>,
) -> Result<String, EvalError> {
    let text = value.to_string();

    let gender = parse_romance_gender(value, "de")?;
    let plural = parse_romance_plural(context);

    // de + article -> contracted form
    let contracted = match (gender, plural) {
        (RomanceGender::Masculine, PluralCategory::One) => "do",      // de + o
        (RomanceGender::Masculine, PluralCategory::Other) => "dos",   // de + os
        (RomanceGender::Feminine, PluralCategory::One) => "da",       // de + a
        (RomanceGender::Feminine, PluralCategory::Other) => "das",    // de + as
    };

    Ok(format!("{} {}", contracted, text))
}
```

### French @le with Elision
```rust
// Source: APPENDIX_STDLIB.md - French elision rules
fn french_le_transform(
    value: &Value,
    context: Option<&Value>,
) -> Result<String, EvalError> {
    let text = value.to_string();

    let gender = parse_romance_gender(value, "le")?;
    let has_vowel = value.has_tag("vowel");
    let plural = parse_romance_plural(context);

    // Elision only in singular
    let article = match (gender, has_vowel, plural) {
        // Elision before vowel (singular only)
        (_, true, PluralCategory::One) => "l'",
        // Normal masculine
        (RomanceGender::Masculine, false, PluralCategory::One) => "le",
        // Normal feminine
        (RomanceGender::Feminine, false, PluralCategory::One) => "la",
        // Plural (no elision, same for both genders)
        (_, _, PluralCategory::Other) => "les",
    };

    // Note: l' has no trailing space (attached via apostrophe)
    if article == "l'" {
        Ok(format!("{}{}", article, text))
    } else {
        Ok(format!("{} {}", article, text))
    }
}
```

### Italian @il with Sound Rules
```rust
// Source: APPENDIX_STDLIB.md - Italian sound-based articles
fn italian_il_transform(
    value: &Value,
    context: Option<&Value>,
) -> Result<String, EvalError> {
    let text = value.to_string();

    let gender = parse_romance_gender(value, "il")?;
    let plural = parse_romance_plural(context);

    // Determine sound category from tags
    let sound = if value.has_tag("vowel") {
        ItalianSound::Vowel
    } else if value.has_tag("s_imp") {
        ItalianSound::SImpura
    } else {
        ItalianSound::Normal
    };

    let article = italian_definite_article(gender, sound, plural);

    // Apostrophe handling for elision
    if article.ends_with('\'') {
        Ok(format!("{}{}", article, text))
    } else {
        Ok(format!("{} {}", article, text))
    }
}

fn italian_definite_article(
    gender: RomanceGender,
    sound: ItalianSound,
    plural: PluralCategory,
) -> &'static str {
    match (gender, sound, plural) {
        // Masculine singular
        (RomanceGender::Masculine, ItalianSound::Normal, PluralCategory::One) => "il",
        (RomanceGender::Masculine, ItalianSound::Vowel, PluralCategory::One) => "l'",
        (RomanceGender::Masculine, ItalianSound::SImpura, PluralCategory::One) => "lo",
        // Masculine plural
        (RomanceGender::Masculine, ItalianSound::Normal, PluralCategory::Other) => "i",
        (RomanceGender::Masculine, ItalianSound::Vowel | ItalianSound::SImpura, PluralCategory::Other) => "gli",
        // Feminine singular
        (RomanceGender::Feminine, ItalianSound::Vowel, PluralCategory::One) => "l'",
        (RomanceGender::Feminine, _, PluralCategory::One) => "la",
        // Feminine plural
        (RomanceGender::Feminine, _, PluralCategory::Other) => "le",
    }
}
```

### TransformRegistry Extension
```rust
// Source: transforms.rs pattern from Phase 6
impl TransformRegistry {
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformKind> {
        // Resolve aliases first
        let canonical = match name {
            // Spanish aliases
            "la" if lang == "es" => "el",
            "una" => "un",
            // French aliases
            "la" if lang == "fr" => "le",
            "une" => "un",
            "au" => "au",  // No alias, distinct transform
            // Portuguese aliases
            "a" if lang == "pt" => "o",
            "uma" => "um",
            // Italian aliases
            "lo" | "la" if lang == "it" => "il",
            "uno" | "una" if lang == "it" => "un",
            // Pass through
            other => other,
        };

        // Language-specific transforms
        match (lang, canonical) {
            // Spanish
            ("es", "el") => Some(TransformKind::SpanishEl),
            ("es", "un") => Some(TransformKind::SpanishUn),
            // French
            ("fr", "le") => Some(TransformKind::FrenchLe),
            ("fr", "un") => Some(TransformKind::FrenchUn),
            ("fr", "de") => Some(TransformKind::FrenchDe),
            ("fr", "au") => Some(TransformKind::FrenchAu),
            // Portuguese
            ("pt", "o") => Some(TransformKind::PortugueseO),
            ("pt", "um") => Some(TransformKind::PortugueseUm),
            ("pt", "de") => Some(TransformKind::PortugueseDe),
            ("pt", "em") => Some(TransformKind::PortugueseEm),
            // Italian
            ("it", "il") => Some(TransformKind::ItalianIl),
            ("it", "un") => Some(TransformKind::ItalianUn),
            ("it", "di") => Some(TransformKind::ItalianDi),
            ("it", "a") => Some(TransformKind::ItalianA),
            _ => None,
        }
    }
}
```

## Article Reference Tables

### Spanish Articles
| Gender | Singular | Plural |
|--------|----------|--------|
| Masculine | el | los |
| Feminine | la | las |

| Gender | Singular | Plural |
|--------|----------|--------|
| Masculine | un | unos |
| Feminine | una | unas |

### French Articles (with elision)
| Gender | Before Consonant | Before Vowel | Plural |
|--------|------------------|--------------|--------|
| Masculine | le | l' | les |
| Feminine | la | l' | les |

| Gender | Singular |
|--------|----------|
| Masculine | un |
| Feminine | une |

### French Contractions
| Preposition | + le | + la | + l' | + les |
|-------------|------|------|------|-------|
| de | du | de la | de l' | des |
| a | au | a la | a l' | aux |

### Portuguese Articles
| Gender | Singular | Plural |
|--------|----------|--------|
| Masculine | o | os |
| Feminine | a | as |

| Gender | Singular |
|--------|----------|
| Masculine | um |
| Feminine | uma |

### Portuguese Contractions
| Preposition | + o | + a | + os | + as |
|-------------|-----|-----|------|------|
| de | do | da | dos | das |
| em | no | na | nos | nas |

### Italian Articles (with sound rules)
| Gender | Normal | Vowel | S-impura | Plural (norm) | Plural (vowel/s-imp) |
|--------|--------|-------|----------|---------------|---------------------|
| Masculine | il | l' | lo | i | gli |
| Feminine | la | l' | - | le | le |

| Gender | Normal | Vowel | S-impura |
|--------|--------|-------|----------|
| Masculine | un | un | uno |
| Feminine | una | un' | una |

### Italian Contractions
| Preposition | + il | + lo | + la | + l' | + i | + gli | + le |
|-------------|------|------|------|------|-----|-------|------|
| di | del | dello | della | dell' | dei | degli | delle |
| a | al | allo | alla | all' | ai | agli | alle |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-language transform implementations | Unified TransformKind enum | Phase 3 | Single dispatch, exhaustive matching |
| Phonetic vowel detection | Tag-based vowel marking | RLF design | Predictable, no edge case failures |
| Separate plural transforms | Context selector for plural | Phase 6 | Reuse pattern for all languages |

**Deprecated/outdated:**
- None for this phase - building on established Phase 6 patterns

## Open Questions

Things that couldn't be fully resolved:

1. **Italian Indefinite Feminine Elision**
   - What we know: "un'" is used before feminine vowel words (un'amica)
   - What's unclear: Should un' have trailing apostrophe in output, requiring special formatting?
   - Recommendation: Handle like French l' - include apostrophe in article string, no trailing space

2. **French @liaison Transform**
   - What we know: APPENDIX_STDLIB mentions @liaison for prevocalic forms (ce/cet, beau/bel)
   - What's unclear: Is this in scope for Phase 7? FR-05 lists it.
   - Recommendation: Include @liaison - it follows the :vowel tag pattern already established

3. **Numeric Context for Plural**
   - What we know: Context can be string (:one/:other) or potentially a number
   - What's unclear: Should `@el:3` produce plural? Or only string contexts?
   - Recommendation: Support both - numeric context uses plural rules (1=one, else=other)

## Sources

### Primary (HIGH confidence)
- docs/APPENDIX_STDLIB.md - Complete specification for all Romance transforms
- docs/DESIGN.md - Transform syntax, context selector behavior
- crates/rlf/src/interpreter/transforms.rs - Existing transform patterns
- .planning/phases/07-romance-language-transforms/07-CONTEXT.md - User decisions

### Secondary (MEDIUM confidence)
- .planning/phases/06-english-and-germanic-transforms/06-RESEARCH.md - Established patterns
- Standard Romance grammar references for article tables

### Tertiary (LOW confidence)
- None - all findings verified with project documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, extending existing infrastructure
- Architecture: HIGH - Patterns from Phase 6 directly apply
- Language rules: HIGH - Article tables are standard grammar, well-documented
- Elision handling: HIGH - Decisions locked in CONTEXT.md

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - stable domain, established patterns)
