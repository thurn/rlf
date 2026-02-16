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

3. **Document this behavior** clearly in `DESIGN.md` with
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

## Proposal 2: Required Explicit Selection

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

## Proposal 3: Body-less `:from` for Transparent Wrappers

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

## Proposal 4: Translation Linter

### Problem

Translation files are written by translators who may not fully understand RLF's
evaluation semantics. Several categories of mistakes produce silently wrong
output or unnecessarily verbose definitions:

- Writing 6-line passthrough variant blocks when a one-line `:from` template
  suffices (addressed by Proposal 1 documentation, but translators may not
  read it).
- Using explicit selectors like `{$base:nom}` inside the `nom:` variant of a
  `:from` block, where bare `{$base}` already resolves to the nominative form.
- Writing `= "{$p}"` without `:from`, silently dropping tags and variants.
- Using `= :from($p) "{$p}"` instead of the shorter `= :from($p);` (Proposal
  4).

A linter integrated into `load_translations()` would catch these patterns
automatically and suggest simpler alternatives. The existing `LoadWarning` enum
and `lint_definitions()` API already provide the infrastructure for this.

### Existing Infrastructure

The AST is explicitly public for tooling:

```rust
// crates/rlf/src/parser/ast.rs
//! These types are public to enable external tooling (linters, formatters, etc.).
```

New lint rules add new `LoadWarning` variants and new validation passes over
the parsed AST.

### Proposed Lint Rules

#### Lint 1: Redundant Passthrough Variant Block (static, AST-only)

**Detects:** A `:from($p)` phrase with a variant block where every entry's
template is `"{$p:KEY} ..."` and KEY matches the variant entry's key name.

**Example trigger:**

```
pred_with_constraint($base, $c) = :from($base) {
    nom: "{$base:nom} {$c}",
    acc: "{$base:acc} {$c}",
    gen: "{$base:gen} {$c}",
    *prep: "{$base:prep} {$c}",
};
```

**Suggested fix:**

```
pred_with_constraint($base, $c) = :from($base) "{$base} {$c}";
```

**Detection algorithm:**

1. Find phrases where `from_param` is `Some(p)` and `body` is
   `PhraseBody::Variants(entries)`.
2. For each variant entry, check if the template contains an interpolation of
   `$p` with a single static selector matching the entry's key.
3. If ALL entries match this pattern (i.e., every entry just passes its own key
   through to `$p`), flag as redundant.
4. Entries where the surrounding text differs between variants are NOT flagged
   (the variant block is doing real work, e.g., adjective agreement).

#### Lint 2: Redundant Explicit Selector in `:from` Context (static, AST-only)

**Detects:** Inside a `:from($p)` variant block, an interpolation `{$p:KEY}`
where KEY matches the enclosing variant entry's key.

**Example trigger:**

```
wrapper($s) = :from($s) {
    nom: "good {$s:nom}",
    acc: "good {$s:acc}",
};
```

**Suggested fix:**

```
wrapper($s) = :from($s) {
    nom: "good {$s}",
    acc: "good {$s}",
};
```

Note: this lint is independent of Lint 1. The variant block may be needed
(because surrounding text differs per entry), but the explicit selector on
`$p` is still redundant within each entry.

**Detection algorithm:**

1. Find phrases with `from_param = Some(p)` and `PhraseBody::Variants`.
2. Within each variant entry, scan template interpolations for
   `Reference::Parameter(p)` with a `Selector::Identifier(key)` where `key`
   matches the enclosing variant entry's key.
3. Flag those selectors as redundant.

#### Lint 3: Likely Missing `:from` (static, AST-only)

**Detects:** A phrase without `:from` and without its own explicit tags, where
a parameter appears in the template body (directly or via a phrase call). This
is a strong signal that the phrase is a compositional wrapper that should
preserve the parameter's grammatical metadata.

**Example triggers:**

```
// Direct parameter reference without :from
wrapper($p) = "{$p}";

// Parameter passed to a phrase call without :from
allied_subtype_plural($t) = "allied {subtype($t):other}";

// Parameter used with selector without :from
subtype_plural($s) = "<b>{$s:other}</b>";
```

**Suggested fix (for each):**

```
wrapper($p) = :from($p);
allied_subtype_plural($t) = :from($t) "allied {subtype($t):other}";
subtype_plural($s) = :from($s) "<b>{$s:other}</b>";
```

**Why:** Without `:from`, the result is a plain string. Tags and variants from
the parameter are lost, which silently breaks downstream variant selection and
transform tag-reading. In the dreamtides codebase, ~10 phrases exhibit this
pattern and are likely bugs.

**Exclusion:** Phrases that define their own explicit tags (`:a`, `:an`,
`:masc`, `:fem`, etc.) are excluded. These phrases deliberately define their
own grammatical identity rather than inheriting from a parameter. In the
dreamtides codebase, every phrase that takes a Phrase parameter and
deliberately omits `:from` has its own tags (e.g.,
`allied_subtype($t) = :an "allied {subtype($t)}"`).

**Detection algorithm:**

1. Find phrases where `from_param` is `None` and `tags` is empty.
2. Scan the template body for any `Reference::Parameter(name)` (direct
   reference) or `Reference::PhraseCall` where an argument is
   `Reference::Parameter(name)` (parameter passed to a phrase call).
3. If any parameter appears in such a position, flag as likely missing `:from`.
4. Skip phrases where `tags` is non-empty (these deliberately define their own
   identity).

#### Lint 4: Verbose Transparent Wrapper (static, AST-only)

**Detects:** `= :from($p) "{$p}"` which can be written as `= :from($p);`.

**Example trigger:**

```
predicate_with_indefinite_article($p) = :from($p) "{$p}";
```

**Suggested fix:**

```
predicate_with_indefinite_article($p) = :from($p);
```

**Detection algorithm:**

1. Find phrases where `from_param = Some(p)` and `body` is
   `PhraseBody::Simple(template)`.
2. Check if the template has exactly one segment, an interpolation of
   `Reference::Parameter(p)` with no selectors and no transforms.
3. If so, suggest body-less `:from`.

#### Lint 5: Missing Selector on Multi-Dimensional Phrase (runtime)

This is Proposal 2 (Required Explicit Selection). Unlike the other lints, this
one runs during evaluation because it requires knowing the runtime `Value` type
of a parameter. See Proposal 2 for details.

#### Lint 6: Phrase Argument Without `:from` (runtime)

**Detects:** A phrase is called with a `Value::Phrase` argument, but the phrase
definition has no `:from`. The result loses the argument's tags and variants.

This is the runtime complement to static Lint 3. While Lint 3 flags suspicious
AST patterns at parse time, Lint 6 catches the actual metadata loss at
evaluation time with full type information.

**Example triggers (all found as likely bugs in dreamtides):**

```
// Tag loss: result of allied_subtype_plural has no tags, so
// downstream {@a allied_subtype_plural($t)} can't read :a/:an
allied_subtype_plural($t) = "allied {subtype($t):other}";

// Variant loss: result of subtype_plural has no variants, so
// downstream {subtype_plural($s):nom} fails
subtype_plural($s) = "<b>{$s:other}</b>";

// Multi-param: n_figments loses $f's tags
n_figments($n, $f) = :match($n) {
    1: "a {figment($f)}",
    *other: "{text_number($n)} {figments_plural($f)}",
};
```

**Detection algorithm:**

1. During phrase evaluation (in the evaluator), after resolving all argument
   values, check if the phrase definition has `from_param = None`.
2. Check if any resolved argument is a `Value::Phrase` (not `Value::Number`,
   `Value::Float`, or `Value::String`).
3. If both conditions hold, emit a warning: the result will be a plain string
   that has lost the Phrase argument's tags and variant structure.
4. The warning identifies the phrase name and the parameter(s) that received
   Phrase values.

**Why runtime, not static:** At parse time, parameter types are unknown — `$t`
could receive a number, a string, or a Phrase depending on the call site. Only
at evaluation time can we confirm that a Phrase value was passed and its
metadata will be lost.

**Interaction with Lint 3:** Lint 3 provides early static feedback at parse
time for obvious cases (no tags, parameter in template). Lint 6 catches all
remaining cases at runtime, including multi-parameter phrases and cases where
the parameter is used indirectly through phrase calls.

### Implementation

**Phase 1: Static lints (Lints 1-4).**

Add a `pub fn lint_definitions(defs: &[PhraseDefinition]) -> Vec<LoadWarning>`
function that operates purely on the parsed AST. This can be called from
`load_translations()` or independently by external tools.

New `LoadWarning` variants:

```rust
/// Variant block on :from phrase could be replaced with simple template.
RedundantPassthroughBlock {
    name: String,
    language: String,
},

/// Explicit selector on :from parameter matches enclosing variant key.
RedundantFromSelector {
    name: String,
    language: String,
    variant_key: String,
    parameter: String,
},

/// Phrase without :from or tags uses a parameter that may carry metadata.
LikelyMissingFrom {
    name: String,
    language: String,
    parameter: String,
},

/// :from phrase with identity template could use body-less form.
VerboseTransparentWrapper {
    name: String,
    language: String,
},
```

**Phase 2: Runtime lints (Lints 5-6).**

Lint 5 (Proposal 2): check in the evaluator during template interpolation for
bare references to Phrases with multi-dimensional variants outside `:from`
context. Produces warnings or errors.

Lint 6: check in the evaluator during phrase call dispatch. When a phrase
without `:from` receives a `Value::Phrase` argument, emit a warning. New
`EvalWarning` type (or extend `LoadWarning`):

```rust
/// Phrase called with Phrase-valued argument but has no :from.
/// Tags and variants from the argument will be lost in the result.
PhraseArgumentWithoutFrom {
    phrase: String,
    parameter: String,
    argument_tags: Vec<String>,
},
```

### Severity

All static lints are **suggestions** (non-blocking). They appear in
`lint_definitions()` output. The runtime lints (Lints 5-6) are configurable
as warning or error.

### Impact

- Catches verbose patterns that Proposal 1 documentation alone may not prevent
- Catches the dangerous missing-`:from` pattern that silently loses metadata
- Gives translators actionable feedback with specific suggested fixes
- Builds on existing `LoadWarning` / `lint_definitions()` infrastructure
- Static lints require no evaluator changes -- purely AST analysis

---

## Implementation Priority

| Proposal | Impact | Complexity | Action |
|----------|--------|------------|--------|
| 1. `:from` passthrough docs | High | None (tests + docs only) | Write tests, update DESIGN.md |
| 2. Required explicit selection | Medium | Low-Medium | Evaluator validation pass |
| 3. Body-less `:from` | Low-Medium | Low | ~10-line parser change per parser |
| 4. Translation linter | Medium | Low-Medium | New `LoadWarning` variants + AST pass |

Recommended order: 1, 3, 2, 4. Proposal 1 requires no code changes and saves
the most lines immediately. Proposal 3 is a small parser change. Proposal 2

adds runtime safety. Proposal 4 reinforces all other proposals by automatically
flagging verbose patterns and missing `:from`.

---

## References

- English phrase definitions: `rules_engine/src/strings/src/strings.rs`
- RLF evaluator: `crates/rlf/src/interpreter/evaluator.rs`
- Value type: `crates/rlf/src/types/value.rs`
- Phrase type: `crates/rlf/src/types/phrase.rs`
- RLF macro parser: `crates/rlf-macros/src/parse.rs`
- Runtime file parser: `crates/rlf/src/parser/file.rs`
- Existing `:from` tests: `crates/rlf/tests/interpreter_eval.rs`
- `:from` + `:match` tests: `crates/rlf/tests/interpreter_match.rs`
- CLDR plural rules: `crates/rlf/src/interpreter/plural.rs`
- Design spec: `docs/DESIGN.md`
