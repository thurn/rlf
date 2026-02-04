# Phase 2: Interpreter Engine - Research

**Researched:** 2026-02-04
**Domain:** Template evaluation engine with variant resolution and phrase lookup
**Confidence:** HIGH

## Summary

This phase implements the core evaluation engine for RLF templates. The interpreter takes parsed AST structures (from Phase 1) and produces formatted strings by resolving phrase references, applying variant selection based on parameters, and substituting values. The implementation follows a classic tree-walking interpreter pattern where templates are evaluated by recursively processing segments.

The standard approach uses a registry-based architecture where phrases are loaded per-language into a `HashMap<String, PhraseDefinition>` structure, with lookup by name or by `PhraseId` hash. Variant resolution uses the CLDR plural rules via `icu_plurals` for numeric selection, and tag-based selection reads phrase metadata. Cycle detection uses a call stack (`Vec<String>`) checked before each phrase resolution.

Key decisions from CONTEXT.md constrain the implementation: `Result<String, EvalError>` return type (no panics in interpreter), error on missing variants (no fallback to 'other'), max depth of 64, and both compile-time and runtime validation paths available.

**Primary recommendation:** Implement a `PhraseRegistry` per language with `HashMap` lookup, an `EvalContext` struct carrying parameters/depth/call-stack, and a recursive `eval_template()` function that processes `Segment` variants.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| icu_plurals | 2.1 | CLDR plural category selection | Official Unicode ICU4X implementation for Rust |
| icu_locale_core | 2.1 | Locale parsing and `locale!` macro | Required by icu_plurals, provides compile-time locale construction |
| thiserror | 2.0 | Error type derivation | Already in project, standard for library error types |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| bon | 3.8 | Builder pattern | Already in project, use for EvalContext/interpreter builders |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| icu_plurals | intl_pluralrules | Older Mozilla crate, less maintained than ICU4X |
| HashMap for registry | IndexMap | Preserves insertion order but not needed for lookup-only |

**Installation:**
```bash
cargo add icu_plurals@2 icu_locale_core@2
```

## Architecture Patterns

### Recommended Project Structure
```
crates/rlf/src/
  interpreter/
    mod.rs           # Public exports
    error.rs         # EvalError type
    registry.rs      # PhraseRegistry (per-language phrase storage)
    context.rs       # EvalContext (evaluation state)
    evaluator.rs     # eval_template(), eval_segment() functions
    plural.rs        # CLDR plural category resolution
```

### Pattern 1: Registry-Based Phrase Storage
**What:** Store parsed phrases in a `HashMap<String, PhraseDefinition>` per language, with a secondary `HashMap<u64, String>` for PhraseId-to-name lookup.
**When to use:** Always - this is the core storage mechanism.
**Example:**
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md
pub struct PhraseRegistry {
    /// Phrases by name
    phrases: HashMap<String, PhraseDefinition>,
    /// PhraseId hash to name mapping (for collision detection and lookup)
    id_to_name: HashMap<u64, String>,
}

impl PhraseRegistry {
    pub fn get(&self, name: &str) -> Option<&PhraseDefinition> {
        self.phrases.get(name)
    }

    pub fn get_by_id(&self, id: u64) -> Option<&PhraseDefinition> {
        let name = self.id_to_name.get(&id)?;
        self.phrases.get(name)
    }

    pub fn insert(&mut self, def: PhraseDefinition) -> Result<(), LoadError> {
        let id = PhraseId::from_name(&def.name);
        // Check for collision
        if let Some(existing) = self.id_to_name.get(&id.as_u64()) {
            if existing != &def.name {
                return Err(LoadError::PhraseIdCollision {
                    name1: existing.clone(),
                    name2: def.name.clone(),
                });
            }
        }
        self.id_to_name.insert(id.as_u64(), def.name.clone());
        self.phrases.insert(def.name.clone(), def);
        Ok(())
    }
}
```

### Pattern 2: Evaluation Context
**What:** Carry evaluation state through recursive calls: parameters, call stack, depth counter.
**When to use:** Pass to all evaluation functions.
**Example:**
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md evaluation algorithm
pub struct EvalContext<'a> {
    /// Parameters available during evaluation
    params: &'a HashMap<String, Value>,
    /// Call stack for cycle detection (phrase names)
    call_stack: Vec<String>,
    /// Current recursion depth
    depth: usize,
    /// Maximum allowed depth (default 64)
    max_depth: usize,
}

impl<'a> EvalContext<'a> {
    pub fn push_call(&mut self, name: &str) -> Result<(), EvalError> {
        if self.depth >= self.max_depth {
            return Err(EvalError::MaxDepthExceeded);
        }
        if self.call_stack.contains(&name.to_string()) {
            return Err(EvalError::CyclicReference {
                chain: self.call_stack.clone(),
            });
        }
        self.call_stack.push(name.to_string());
        self.depth += 1;
        Ok(())
    }

    pub fn pop_call(&mut self) {
        self.call_stack.pop();
        self.depth -= 1;
    }
}
```

### Pattern 3: Tree-Walking Evaluator
**What:** Recursively process AST nodes, building output string.
**When to use:** Core evaluation logic.
**Example:**
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md evaluation algorithm
fn eval_template(
    template: &Template,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    lang: &str,
) -> Result<String, EvalError> {
    let mut output = String::new();
    for segment in &template.segments {
        match segment {
            Segment::Literal(s) => output.push_str(s),
            Segment::Interpolation { transforms, reference, selectors } => {
                // 1. Resolve reference to Value
                let value = resolve_reference(reference, ctx, registry, lang)?;
                // 2. Apply selectors to get variant
                let selected = apply_selectors(value, selectors, ctx)?;
                // 3. Apply transforms (right-to-left) - Phase 3
                let result = selected.to_string();
                output.push_str(&result);
            }
        }
    }
    Ok(output)
}
```

### Pattern 4: Variant Fallback Resolution
**What:** Try exact key, then progressively shorter keys by removing trailing dot-segments.
**When to use:** All variant lookups.
**Example:**
```rust
// Source: APPENDIX_RUST_INTEGRATION.md selection evaluation
fn resolve_variant<'a>(
    variants: &'a HashMap<VariantKey, String>,
    key: &str,
) -> Option<&'a str> {
    // Try exact match
    if let Some(v) = variants.get(&VariantKey::new(key)) {
        return Some(v);
    }
    // Try progressively shorter keys
    let mut current = key;
    while let Some(dot_pos) = current.rfind('.') {
        current = &current[..dot_pos];
        if let Some(v) = variants.get(&VariantKey::new(current)) {
            return Some(v);
        }
    }
    None
}
```

### Anti-Patterns to Avoid
- **Cloning AST on each evaluation:** Pass references, don't clone the template for each call.
- **Global mutable state:** Use `&mut EvalContext` passed through call chain, not global state.
- **String concatenation in loop without capacity hint:** Use `String::with_capacity()` when output size is estimable.
- **Silent fallback to 'other':** Per CONTEXT.md, missing variant is an error.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLDR plural rules | Custom plural logic | `icu_plurals::PluralRules` | 200+ languages with complex rules (Arabic has 6 categories) |
| Locale parsing | String matching | `icu_locale_core::locale!` | Compile-time validation, proper subtag handling |
| Error type Display | Manual impl | `thiserror::Error` derive | Consistent formatting, source chaining |

**Key insight:** Plural rules are the #1 thing to not hand-roll. English (one/other) is trivial but Russian (one/few/many/other), Arabic (zero/one/two/few/many/other), and others have complex modular arithmetic rules that ICU4X implements correctly.

## Common Pitfalls

### Pitfall 1: Parameter vs Phrase Resolution Ambiguity
**What goes wrong:** At parse time, `{name}` could be a parameter or phrase reference. Wrong resolution breaks evaluation.
**Why it happens:** AST uses `Reference::Identifier` which defers distinction to runtime.
**How to avoid:** Resolver checks parameters first, then phrase registry. Document order clearly.
**Warning signs:** Tests pass with simple cases but fail when parameter shadows phrase name.

### Pitfall 2: Selector Context Confusion
**What goes wrong:** Confusing transform context (`:acc` in `@der:acc`) with variant selector (`:one` in `card:one`).
**Why it happens:** Both use `:` syntax but have different semantics.
**How to avoid:** Transform context is consumed by transform execution (Phase 3). Selectors are consumed by variant resolution. Process in correct order: resolve reference, apply selectors, then apply transforms.
**Warning signs:** Transform gets variant key instead of context parameter.

### Pitfall 3: Depth vs Cycle Detection Overlap
**What goes wrong:** Implementing both but not understanding when each triggers.
**Why it happens:** Both prevent infinite loops but for different reasons.
**How to avoid:** Cycle detection catches `a -> b -> a` (same phrase twice in stack). Depth limit catches `a -> b -> c -> d -> ...` (64 unique phrases). Both can trigger.
**Warning signs:** Cycle test passes but depth test fails, or vice versa.

### Pitfall 4: Multi-Dimensional Key Construction
**What goes wrong:** Building key `"nom.one"` incorrectly when multiple selectors chain.
**Why it happens:** Selectors can be literal (`one`) or parameter-derived (CLDR category from number).
**How to avoid:** Build key by joining selector results with `.` in order. `{card:acc:n}` with n=1 in English -> `"acc.one"`.
**Warning signs:** Variant lookup fails for multi-dimensional keys.

### Pitfall 5: Tag-Based Selection with Missing Tags
**What goes wrong:** Phrase has `:fem` tag, selecting phrase has no `fem` variant.
**Why it happens:** Tag-based selection uses first tag of selector phrase as variant key.
**How to avoid:** Return `MissingVariant` error with helpful message including available variants.
**Warning signs:** Selection on phrase with tag errors unhelpfully.

## Code Examples

Verified patterns from design documents:

### CLDR Plural Category Resolution
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};
use icu_locale_core::locale;

fn plural_category(lang: &str, n: i64) -> &'static str {
    let loc = match lang {
        "en" => locale!("en"),
        "ru" => locale!("ru"),
        "ar" => locale!("ar"),
        "de" => locale!("de"),
        "es" => locale!("es"),
        "fr" => locale!("fr"),
        "ja" => locale!("ja"),
        "zh" => locale!("zh"),
        _ => locale!("en"),  // fallback
    };

    let rules = PluralRules::try_new(loc.into(), PluralRuleType::Cardinal)
        .expect("locale should be supported");

    match rules.category_for(n) {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}
```

### Selector Resolution
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md selector resolution
fn resolve_selector(
    selector: &Selector,
    value: &Value,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<String, EvalError> {
    match selector {
        Selector::Identifier(name) => {
            // Check if it's a parameter reference
            if let Some(param_value) = ctx.params.get(name) {
                // Number -> CLDR category
                if let Some(n) = param_value.as_number() {
                    return Ok(plural_category(lang, n).to_string());
                }
                // Phrase -> first tag
                if let Some(phrase) = param_value.as_phrase() {
                    if let Some(tag) = phrase.first_tag() {
                        return Ok(tag.to_string());
                    }
                    return Err(EvalError::MissingTag {
                        phrase: name.clone(),
                        expected: vec!["any tag".to_string()],
                    });
                }
                // String -> use literally or parse as number
                if let Some(s) = param_value.as_string() {
                    if let Ok(n) = s.parse::<i64>() {
                        return Ok(plural_category(lang, n).to_string());
                    }
                    return Ok(s.to_string());
                }
            }
            // Not a parameter -> literal selector key
            Ok(name.clone())
        }
    }
}
```

### EvalError Definition
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md EvalError
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("phrase not found: '{name}'")]
    PhraseNotFound { name: String },

    #[error("phrase not found for id: {id}")]
    PhraseNotFoundById { id: u64 },

    #[error("missing variant '{key}' in phrase '{phrase}', available: {}", available.join(", "))]
    MissingVariant {
        phrase: String,
        key: String,
        available: Vec<String>,
    },

    #[error("transform '@{transform}' requires tag {expected:?} on phrase '{phrase}'")]
    MissingTag {
        transform: String,
        expected: Vec<String>,
        phrase: String,
    },

    #[error("phrase '{phrase}' expects {expected} arguments, got {got}")]
    ArgumentCount {
        phrase: String,
        expected: usize,
        got: usize,
    },

    #[error("cyclic reference detected: {}", chain.join(" -> "))]
    CyclicReference { chain: Vec<String> },

    #[error("maximum recursion depth exceeded")]
    MaxDepthExceeded,
}
```

### Reference Resolution
```rust
// Source: APPENDIX_RUNTIME_INTERPRETER.md evaluation algorithm
fn resolve_reference(
    reference: &Reference,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    lang: &str,
) -> Result<Value, EvalError> {
    match reference {
        Reference::Identifier(name) => {
            // Parameters take precedence
            if let Some(value) = ctx.params.get(name) {
                return Ok(value.clone());
            }
            // Then try phrase lookup
            if let Some(def) = registry.get(name) {
                ctx.push_call(name)?;
                let phrase = eval_phrase_def(def, &HashMap::new(), ctx, registry, lang)?;
                ctx.pop_call();
                return Ok(Value::Phrase(phrase));
            }
            Err(EvalError::PhraseNotFound { name: name.clone() })
        }
        Reference::PhraseCall { name, args } => {
            // Resolve each argument
            let resolved_args: Vec<Value> = args
                .iter()
                .map(|arg| resolve_reference(arg, ctx, registry, lang))
                .collect::<Result<Vec<_>, _>>()?;

            // Look up phrase
            let def = registry
                .get(name)
                .ok_or_else(|| EvalError::PhraseNotFound { name: name.clone() })?;

            // Check argument count
            if def.parameters.len() != resolved_args.len() {
                return Err(EvalError::ArgumentCount {
                    phrase: name.clone(),
                    expected: def.parameters.len(),
                    got: resolved_args.len(),
                });
            }

            // Build parameter map
            let params: HashMap<String, Value> = def
                .parameters
                .iter()
                .zip(resolved_args)
                .map(|(name, value)| (name.clone(), value))
                .collect();

            ctx.push_call(name)?;
            let phrase = eval_phrase_def(def, &params, ctx, registry, lang)?;
            ctx.pop_call();
            Ok(Value::Phrase(phrase))
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| intl_pluralrules | icu_plurals (ICU4X) | 2023+ | Unicode consortium's official Rust implementation |
| Custom locale strings | icu_locale_core | 2023+ | Type-safe locale handling with compile-time validation |
| error-chain | thiserror | 2020+ | Simpler, lighter weight, better ergonomics |

**Deprecated/outdated:**
- `intl_pluralrules`: While functional, `icu_plurals` from ICU4X is now the canonical choice.
- Manual `Display` impl for errors: Use `thiserror` derive macro.

## Open Questions

Things that couldn't be fully resolved:

1. **Scope inheritance for nested phrase calls**
   - What we know: CONTEXT.md marks this as Claude's discretion
   - What's unclear: Should `foo(x) = "{bar}"` where `bar` references `{x}` see the parent's `x`?
   - Recommendation: No inheritance - each phrase call gets fresh scope with only its declared parameters. Simpler, more predictable. If parent wants to pass value, it must do so explicitly: `foo(x) = "{bar(x)}"`.

2. **Transform execution timing (Phase 3 boundary)**
   - What we know: This phase handles variant selection, Phase 3 handles transforms
   - What's unclear: How to structure code so transforms can be added cleanly
   - Recommendation: Design `eval_interpolation` to return `(Value, Vec<Transform>)` tuple. Phase 3 will add `apply_transforms()` call.

3. **PluralRules caching strategy**
   - What we know: Creating `PluralRules` is not free (loads CLDR data)
   - What's unclear: Cache per-language? Per-interpreter? Global static?
   - Recommendation: Cache in interpreter struct, one per language used. `HashMap<String, PluralRules>`. Lazy initialization.

## Sources

### Primary (HIGH confidence)
- docs/DESIGN.md - RLF language specification (local file)
- docs/APPENDIX_RUNTIME_INTERPRETER.md - Interpreter API and evaluation rules (local file)
- docs/APPENDIX_RUST_INTEGRATION.md - Rust types and error handling (local file)
- [icu_plurals docs.rs](https://docs.rs/icu_plurals/latest/icu_plurals/) - Version 2.1.1, PluralRules API
- [icu_locale_core docs.rs](https://docs.rs/icu_locale_core/latest/icu_locale_core/) - Version 2.1.1, locale! macro

### Secondary (MEDIUM confidence)
- [Rust Design Patterns - Interpreter](https://rust-unofficial.github.io/patterns/patterns/behavioural/interpreter.html) - Tree-walking pattern
- [Building fast interpreters in Rust - Cloudflare](https://blog.cloudflare.com/building-fast-interpreters-in-rust/) - Performance patterns
- [thiserror crate](https://docs.rs/thiserror) - Error derivation

### Tertiary (LOW confidence)
- [GeeksforGeeks - Cycle Detection](https://www.geeksforgeeks.org/dsa/detect-cycle-in-a-graph/) - Algorithm reference

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - ICU4X is the official Unicode Rust implementation
- Architecture: HIGH - Patterns derived directly from DESIGN.md appendices
- Pitfalls: MEDIUM - Based on interpreter design experience, not RLF-specific testing

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - stable domain, well-specified by design docs)
