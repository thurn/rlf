# RLF Syntax Proposals for Inflected Language Support

## Motivation

RLF phrase composition works well for analytic languages (English, Chinese) where
words don't change form based on grammatical context. For **inflected languages**
(Russian, German, Arabic, Turkish, Polish, Finnish, Hungarian), translation files
can become verbose because:

- Nouns must be defined with 6-12+ variant forms (case x number).
- Every composition phrase must manually propagate case selectors to inner
  parameters.
- Every verb phrase must remember to annotate which case it governs on each
  argument.
- Pass-through wrapper phrases require explicit `:from` machinery even when
  they're semantically no-ops.

These four proposals address these pain points. They are ordered by
implementation priority.

---

## Proposal 1: Document That `:from` Already Handles Passthrough

### Problem

Several Russian translation phrases contain verbose variant blocks that
manually pass case selectors through to an inner parameter:

```
pred_with_constraint($base, $constraint) = :from($base) {
    nom: "{$base:nom} {$constraint}",
    acc: "{$base:acc} {$constraint}",
    gen: "{$base:gen} {$constraint}",
    dat: "{$base:dat} {$constraint}",
    inst: "{$base:inst} {$constraint}",
    *prep: "{$base:prep} {$constraint}",
};
```

This pattern recurs in roughly 20-25 phrases: `pred_with_constraint`,
`allied_pred`, `enemy_pred`, `in_your_void`, `another_pred`, `non_subtype`,
`fast_predicate`, `allied_subtype`, `enemy_subtype`, `other_pred_plural`, and
more. Each produces 6 nearly-identical lines for Russian.

### Finding: `:from` Already Handles This

The evaluator's `eval_with_from_modifier()` path already iterates over the
`:from` parameter's variants and binds the parameter to each variant's text as
the default. This means bare `{$base}` in a simple `:from` template resolves
to the correct case variant automatically.

The verbose form above is equivalent to:

```
pred_with_constraint($base, $constraint) = :from($base) "{$base} {$constraint}";
```

When called as `pred_with_constraint(enemy(), constraint):acc`:

1. `:from($base)` iterates over `enemy()`'s variants
2. For the `acc` variant: `$base` is bound to a Phrase where the default text
   is the accusative text of `enemy()`
3. `{$base}` in the template resolves to this accusative text
4. The result Phrase has an `acc` variant with the composed text

Similarly, `:from` + `:match` also correctly propagates variant context. The
`eval_from_with_match()` path evaluates match branches within each inherited
variant context:

```
count_pred($n, $base) = :from($base) :match($n) {
    1: "{$base}",
    *other: "{$n} {$base}",
};
```

Here `{$base}` in each match branch resolves to the correct case form.

The only case where a variant block is genuinely needed is when the **wrapper
text itself** changes per case (e.g., Russian adjective agreement where
"вражеский" declines alongside the noun). That is the variant block doing real
work, not passthrough.

### Action Items

1. **Write tests** confirming that `= :from($p) "{$p} extra"` produces correct
   per-variant output without explicit selectors. Existing test
   `eval_from_modifier_inherits_tags` implicitly covers this (verifying
   `variant("one")` and `variant("other")` produce correct results) but a more
   explicit test with case-like variant names (nom, acc, gen) would make the
   behavior undeniable.

2. **Write a test** showing the composition chain: a term with case variants
   passed through a `:from` phrase, then selected by a consumer phrase with
   `:acc`, produces the correct accusative form.

3. **Document this behavior** clearly in `APPENDIX_RUSSIAN_TRANSLATION.md` with
   before/after examples showing that translators can replace 6-line variant
   blocks with single-line `:from` templates.

4. **Remove verbose passthrough blocks** from existing translation files (once
   tests confirm the simple form works).

### Impact

- **~130 lines saved** in a Russian translation file (20-25 phrases x 6 lines
  reduced to 1 line each).
- No new syntax, no new concepts, no implementation work beyond tests and docs.
- Identical savings for German (4 cases), Polish (7 cases), Finnish (15 cases).

---

## Proposal 2: Value-Returning Transforms

### Problem

Russian has ~40 game nouns (enemy, ally, character, card, event, 23 character
subtypes, figment types). Each requires 12 forms (6 cases x 2 numbers). That's
~480 hand-written forms. Most Russian nouns follow one of ~4 declension
patterns. "Враг" (enemy), "Воин" (Warrior), "союзник" (ally) all decline
identically -- the only difference is the stem.

One of RLF's design principles is to provide a large standard library of
transforms with per-language knowledge. In principle, a transform like
`@decline:masc_anim_hard` could generate all 12 forms from a stem. However,
the current transform architecture cannot do this because **transforms return
`String`, not `Value`**, so they cannot generate new Phrase structures with
variant maps.

### Current Architecture

```
apply_transforms() iterates right-to-left:
  1. First transform receives Value::Phrase (with tags and variants)
  2. Transform executes, returns String
  3. Result wrapped as Value::String for next transform
  4. Tags and variants are LOST after first transform
```

Key signatures:

```rust
// evaluator.rs:796
fn apply_transforms(...) -> Result<String, EvalError>

// transforms.rs:142
pub fn execute(&self, value: &Value, context: Option<&Value>, lang: &str)
    -> Result<String, EvalError>
```

### Proposed Architecture

Change transforms to return `Value` instead of `String`. This enables a
**two-tier** system where some transforms generate new Phrase values (with
variants and tags) while others modify text as before.

**Step 1: Change `TransformKind::execute()` return type.**

```rust
// transforms.rs -- new signature
pub fn execute(&self, value: &Value, context: Option<&Value>, lang: &str)
    -> Result<Value, EvalError>
```

Existing text-only transforms wrap their `String` result as `Value::String`
before returning. This is backward-compatible -- all existing transforms
continue to work with a one-line change per implementation.

**Step 2: Change `apply_transforms()` to thread `Value` through.**

```rust
// evaluator.rs -- new implementation
fn apply_transforms(
    initial_value: &Value,
    transforms: &[Transform],
    ...
) -> Result<Value, EvalError> {
    let mut current = initial_value.clone();
    for transform in transforms.iter().rev() {
        let kind = transform_registry.get(&transform.name, lang)?;
        let ctx_val = resolve_transform_context(&transform.context, ctx)?;
        current = kind.execute(&current, ctx_val.as_ref(), lang)?;
    }
    Ok(current)
}
```

**Step 3: Update callers** to convert `Value` to `String` at the output
boundary only.

```rust
// In eval_template(), where transform results are pushed to output:
let transformed = apply_transforms(&selected, transforms, ...)?;
output.push_str(&transformed.to_string());
```

### What This Enables

**Generative transforms** that return `Value::Phrase` with variant maps:

```
// Hypothetical Russian translation file
enemy = @decline:masc_anim_hard "враг";
```

The `@decline` transform would:
1. Read the stem text ("враг") from `value.to_string()`
2. Read the declension class ("masc_anim_hard") from the context parameter
3. Apply Russian morphology rules to generate all case/number forms
4. Return `Value::Phrase(Phrase { text: "враг", variants: { nom.one: "враг",
   gen.one: "врага", acc.one: "врага", ... }, tags: ["masc", "anim"] })`

**Tags preserved through chaining.** Since transforms now pass `Value`
through, a downstream transform can still read tags set by an upstream
transform or by the original Phrase. This fixes the current limitation where
`{@cap @a card}` loses tags after `@a` executes.

**Article transforms could return Phrase.** German `@der` currently returns a
string like "den". If it returned a Phrase, it could carry case/gender variants
for downstream composition.

### Complexity Assessment

| Change | Files Affected | Risk |
|--------|---------------|------|
| `execute()` return type | transforms.rs | Low -- mechanical change |
| `apply_transforms()` return type | evaluator.rs | Low -- one function |
| Caller updates | evaluator.rs | Low -- add `.to_string()` at output |
| Existing transform implementations | transforms.rs | Low -- wrap returns in `Value::String()` |
| New generative transforms | transforms.rs + semantics | Medium -- new per-language code |

The architecture change itself is low-risk. The per-language morphology
implementations are the real effort, but they can be added incrementally (one
language at a time, one declension class at a time).

### Design Note

An alternative is a parse-time `:pattern` macro system that expands templates
into variant blocks before evaluation. This is simpler to implement but
language-agnostic -- it shifts morphological knowledge to the translator. The
Value-returning transform approach aligns better with RLF's principle of
encoding language knowledge in the standard library so translators don't need
to be linguists.

Both approaches could coexist: `:pattern` for irregular forms, `@decline` for
regular paradigms.

---

## Proposal 3: Required Explicit Selection

### Problem

When a translator writes `{$target}` and the parameter holds a Phrase with
case variants (nom, acc, gen, ...), the system silently uses the default
variant (typically nominative). This produces grammatically wrong text like
"развеять враг" instead of "развеять врага" (accusative). The translator
meant to write `{$target:acc}` but forgot.

This is the most common translation error for inflected languages, and it
produces **silently wrong output** rather than an error.

### Current Usage Analysis (dreamtides strings.rs)

| Reference pattern | Count | Notes |
|-------------------|-------|-------|
| Bare `{$param}` with numeric value | ~45 | Never need selection |
| Bare `{$param}` with string value | ~5 | Never need selection |
| Bare `{$param}` Phrase in `:from` context | ~20 | Default is correct (`:from` binds variant) |
| Bare `{$param}` Phrase outside `:from` | ~5 | Potentially dangerous |
| Bare `{term}` without variants | ~100 | Never need selection |
| Bare `{term}` with variants | ~20 | Relies on default |
| Explicit `{...:key}` selectors | ~30 | Already correct |

The dangerous category -- Phrase parameters referenced without a selector
outside `:from` context -- is small (~5 instances). Requiring explicit
selection here would catch real bugs with minimal disruption.

### Proposed Behavior

**In translation files** (`.rlf` runtime files), produce a **warning** when:

1. A parameter reference `{$param}` has no selector, AND
2. The parameter's value is a Phrase with multi-dimensional variants (i.e.,
   variants with dot-separated keys like `nom.one`, `acc.few`), AND
3. The reference is NOT inside a `:from` variant evaluation context for that
   parameter

This catches exactly the dangerous case (forgetting `:acc` on a case-inflected
noun) while allowing:

- Bare numeric/string parameters (no variants to select from)
- Bare Phrase parameters in `:from` context (`:from` binds the correct variant)
- Bare references to terms with simple variants like `{one, other}` (these are
  typically plurals, not case-inflected, and the default is usually correct)

**In source files** (`rlf!` macro), this validation is not needed because the
source language (English) does not use multi-dimensional case variants.

### Explicit Default Syntax

For cases where the translator intentionally wants the default variant of a
multi-dimensional Phrase, provide an explicit escape hatch:

```
{$target:*}    // "I know this has case variants; give me the default"
```

This makes the intent visible and distinguishes "I want the default" from "I
forgot to select a case."

### Implementation

The check happens in the evaluator during template interpolation. When
resolving a `{$param}` reference:

1. Look up the parameter's `Value`
2. If it's a `Value::Phrase` with any dot-separated variant keys
3. And no selector was provided
4. And the current evaluation is not inside a `:from` context for this parameter
5. Emit a warning (or error, configurable)

This requires threading a "`:from` context set" through the evaluator, tracking
which parameters are currently bound in a `:from` iteration. The evaluator
already tracks this implicitly (via the variant binding in
`eval_with_from_modifier`), so making it explicit is straightforward.

### Impact

- Catches the most common inflected-language translation error at runtime
- Minimal disruption to existing English source files (~0 changes needed)
- Small number of translation file updates (add `:*` to intentional defaults)
- Makes governed case visible at use sites, improving translation readability

---

## Proposal 4: Body-less `:from` for Transparent Wrappers

### Problem

Some phrases exist in the English source to apply English-specific grammar
(articles, pluralization) but are semantic no-ops in many target languages.
`predicate_with_indefinite_article` applies English "a"/"an" -- Russian,
Chinese, Japanese, Korean, and Turkish have no indefinite articles.

Currently, the translator must write:

```
predicate_with_indefinite_article($p) = :from($p) "{$p}";
```

This requires understanding `:from` semantics just to express "do nothing."
Worse, if a translator omits `:from`, the phrase silently drops variant and tag
information, causing downstream case selection to fail.

### Proposed Syntax

Allow `:from` without a template body:

```
predicate_with_indefinite_article($p) = :from($p);
```

**Semantics:** The phrase returns its argument unchanged -- same text, same
tags, same variant structure. Equivalent to `:from($p) "{$p}"` but with
explicit intent. No template is needed.

If a phrase has multiple parameters, `:from($p)` specifies which parameter is
the identity target. The other parameters are unused (already supported by
RLF -- unused parameters don't cause errors).

### Implementation

In both parsers (macro `parse.rs` and runtime `file.rs`), when `:from($param)`
is followed by `;` instead of a template string or variant block, treat it as
syntactic sugar for `:from($param) "{$param}"`.

This is a ~10-line parser change in each parser. No evaluator changes needed.

### Impact

- **~10 lines saved** per translation file.
- **Clarity:** `= :from($p);` communicates "this phrase doesn't apply in my
  language" far more clearly than `= :from($p) "{$p}"`.
- **Safety:** A translator who forgets `:from` and writes just `= "{$p}";`
  will silently lose variant information. The body-less form makes the safe
  behavior the easy behavior.
- Applies to 5-6 phrases in every non-English translation (article application,
  English-specific plural wrappers, vowel-based article selection).

### Real-World Example

The card "Abyssal Plunge" produces `dissolve an enemy with cost 3 or more`. The
predicate "an enemy with cost 3 or more" flows through
`predicate_with_indefinite_article` to add the "an". In Russian, this wrapper
must be transparent -- the compound predicate's case variants must pass through
intact so that `dissolve_target` can select accusative on the final result. With
body-less `:from`, the translator writes one self-documenting line and the
variant chain is preserved.

---

## Implementation Priority

| Proposal | Impact | Complexity | Action |
|----------|--------|------------|--------|
| 1. `:from` passthrough docs | High | None (tests + docs only) | Write tests, update translation appendix |
| 2. Value-returning transforms | High (future) | Medium | Architecture change, then incremental |
| 3. Required explicit selection | Medium | Low-Medium | Evaluator validation pass |
| 4. Body-less `:from` | Low-Medium | Low | ~10-line parser change per parser |

Recommended order: 1, 4, 3, 2. Proposal 1 requires no code changes and saves
the most lines immediately. Proposal 4 is a small parser change. Proposal 3
adds safety. Proposal 2 is the biggest architectural investment but enables the
most powerful future capabilities.

---

## References

- English phrase definitions: `rules_engine/src/strings/src/strings.rs`
- RLF evaluator: `crates/rlf/src/interpreter/evaluator.rs`
- Transform execution: `crates/rlf/src/interpreter/transforms.rs`
- Transform registry: `crates/rlf-semantics/src/lib.rs`
- Value type: `crates/rlf/src/types/value.rs`
- Phrase type: `crates/rlf/src/types/phrase.rs`
- RLF macro parser: `crates/rlf-macros/src/parse.rs`
- Runtime file parser: `crates/rlf/src/parser/file.rs`
- Existing `:from` tests: `crates/rlf/tests/interpreter_eval.rs`
- `:from` + `:match` tests: `crates/rlf/tests/interpreter_match.rs`
- CLDR plural rules: `crates/rlf/src/interpreter/plural.rs`
- Design spec: `docs/DESIGN.md`
