# RLF Syntax Proposals for Inflected Language Support

## Motivation

RLF phrase composition works well for analytic languages (English, Chinese) where
words don't change form based on grammatical context. For **inflected languages**
(Russian, German, Arabic, Turkish, Polish, Finnish, Hungarian), the translation
file author faces significant boilerplate because:

- Nouns must be defined with 6-12+ variant forms (case × number).
- Every composition phrase must manually propagate case selectors to inner
  parameters.
- Every verb phrase must remember to annotate which case it governs on each
  argument.
- Pass-through wrapper phrases require explicit `:from` machinery even when
  they're semantically no-ops.

These four proposals address these pain points. They are ordered by impact.

---

## Proposal 1: Case Passthrough (`:passthrough`)

### Problem

When a phrase composes two sub-phrases and the outer phrase's case should
propagate to an inner parameter, the translator must write a variant block that
repeats the same template for every case. This is the single largest source of
boilerplate in inflected translations.

**Current Russian translation of `pred_with_constraint`:**

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

Six lines that all say the same thing: "pass the requested case through to
`$base`." The `$constraint` parameter is case-invariant (Russian prepositions
fix the case of their argument internally), so only `$base` varies.

This pattern recurs in roughly 20-25 phrases: `predicate_with_indefinite_article`,
`subtype`, `subtype_plural`, `allied_pred`, `enemy_pred`, `in_your_void`,
`another_pred`, `non_subtype`, `fast_predicate`, `allied_subtype`,
`enemy_subtype`, `other_pred_plural`, `non_subtype_enemy`, and more. Each
produces 6 nearly-identical lines for Russian, 4 for German, and similar counts
for other case languages.

### Proposed Syntax

```
pred_with_constraint($base, $constraint) = :from($base) :passthrough($base) "{$base} {$constraint}";
```

**Semantics:** When the caller evaluates this phrase with a variant selector
(e.g., `{pred_with_constraint(...):acc}`), the system substitutes the same
selector on all references to the passthrough parameter. So `{$base}` becomes
`{$base:acc}` automatically.

If a specific reference needs a different selector, an explicit annotation
overrides the passthrough: `{$base:nom}` would select nominative regardless of
the outer request.

Multiple parameters can be passthrough targets:
`= :passthrough($a, $b) "{$a} and {$b}"`.

### Justification

- **~130 lines saved** in a Russian translation file (20-25 phrases × 6 lines
  reduced to 1 line each).
- Identical savings for German (4 cases), Polish (7 cases), Finnish (15 cases),
  Turkish (6 cases), Arabic (3 cases), and Hungarian (18 cases).
- The pattern is mechanical and error-prone: a translator who adds a new case
  variant to a noun must also update every composition phrase that wraps it.
  `:passthrough` eliminates this coupling.

### Real-World Example

The card "Scorched Reckoning" produces `dissolve an enemy with spark 3 or more`.
The predicate "enemy with spark 3 or more" is built by the Rust serializer as:

```
pred_with_constraint(enemy(), with_spark_constraint("or more", 3))
```

In Russian, `dissolve_target` needs the accusative of this compound predicate.
Without `:passthrough`, the translator must ensure `pred_with_constraint` has an
`acc` variant block that passes `acc` to `$base`. With `:passthrough`, the case
flows through automatically: `dissolve_target` requests `:acc` →
`pred_with_constraint` passes `:acc` to `enemy()` → correct form "врага".

---

## Proposal 2: Declension Pattern Macros (`:pattern`)

### Problem

Russian has ~40 game nouns (enemy, ally, character, card, event, 23 character
subtypes, figment types). Each requires 12 forms (6 cases × 2 numbers). That's
~480 hand-written forms, where a typo in any single form silently produces wrong
grammar at runtime.

Most Russian nouns follow one of ~4 declension patterns. "Враг" (enemy), "Воин"
(Warrior), "союзник" (ally) all decline identically — the only difference is the
stem. Yet each must independently list all 12 forms.

### Proposed Syntax

Define a pattern once, then apply it by stem:

```
:pattern masc_anim_hard($stem, $pl) = :masc :anim {
    nom.one: "{$stem}",       nom.other: "{$pl}и",
    gen.one: "{$stem}а",      gen.other: "{$pl}ов",
    acc.one: "{$stem}а",      acc.other: "{$pl}ов",
    dat.one: "{$stem}у",      dat.other: "{$pl}ам",
    inst.one: "{$stem}ом",    inst.other: "{$pl}ами",
    *prep.one: "{$stem}е",    prep.other: "{$pl}ах",
};

enemy = :pattern masc_anim_hard("враг", "враг");
warrior = :pattern masc_anim_hard("Воин", "Воин");
ally = :pattern masc_anim_hard("союзник", "союзник");
```

Three lines each instead of twelve.

**Semantics:** `:pattern` declarations define a reusable variant template
parameterized by string arguments. Applying a pattern expands it at parse time
into the equivalent multi-dimensional variant block. The pattern is pure
syntactic sugar — it produces the same Phrase structure as writing the variants
by hand.

Patterns can carry tags (`:masc`, `:anim`) and a default marker (`*`). The
applying phrase can add additional tags: `ally = :pattern masc_anim_hard(...)`
inherits `:masc :anim` from the pattern but could add more.

### Justification

- **~350 lines saved** across a Russian translation file.
- **Error reduction:** A declension bug fixed in the pattern definition
  propagates to all nouns using it. Without patterns, fixing "accusative animate
  = genitive" requires updating every animate masculine noun independently.
- Russian has 3-4 productive patterns, German has ~5, Arabic has ~10 broken
  plural patterns. Each language benefits proportionally.
- The most common translation error for inflected languages is an incorrect case
  form for one variant of one noun. Patterns make this structurally impossible
  for regular nouns.

### Real-World Example

The card "Fury of the Clan" produces `dissolve an enemy with cost less than the
number of allied Warriors`. This requires both "enemy" (враг) and "Warrior"
(Воин) to have full case paradigms. Both are masculine animate 2nd-declension
nouns. Without patterns, that's 24 hand-written forms across two nouns, all
following the same rules. With patterns, each noun is one line referencing the
shared pattern.

### Design Note

An alternative lighter approach is a built-in `@inflect($stem, $class)`
transform that generates forms programmatically based on language-specific
morphology rules. This is more powerful (handles irregular stems, consonant
mutations) but harder to implement and requires per-language morphology code in
the RLF interpreter. The `:pattern` macro approach is language-agnostic — it's
just template expansion.

---

## Proposal 3: Parameter-Level Case Governance (`:gov`)

### Problem

In Russian, each verb governs a specific grammatical case on its direct object.
"Развеять" (dissolve) governs accusative. "Пожертвовать" (abandon) governs
instrumental. "Получить контроль над" (gain control of) governs instrumental.
The translator must remember to write the correct case selector every time a
parameter appears in a template:

```
dissolve_target($target) = "{dissolve} {$target:acc}";
banish_target($target) = "{banish} {$target:acc}";
abandon_target($target) = "{abandon} {$target:inst}";
gain_control_of($target) = "получить контроль над {$target:inst}";
```

If a translator writes `{$target}` and forgets the `:acc`, the phrase silently
uses the default variant (likely nominative), producing grammatically wrong text
like "развеять враг" instead of "развеять врага".

### Proposed Syntax

```
dissolve_target($target:acc) = "{dissolve} {$target}";
abandon_target($target:inst) = "{abandon} {$target}";
```

**Semantics:** The `:acc` annotation on `$target` in the parameter list declares
a default variant selector. Everywhere `$target` appears in the template without
an explicit selector, the system applies `:acc`. An explicit annotation like
`{$target:nom}` overrides the default.

This is purely a translation-file feature — the Rust API signature remains
`dissolve_target($target)`. The governance annotation exists only in the
translation file's phrase definition.

### Justification

- **Error prevention** is the primary benefit, not line-count savings (~50 `:acc`
  annotations removed). Forgetting a case selector is the most likely translator
  mistake, and it produces silently wrong output rather than a compile-time error.
- Makes the governed case **declarative and visible** at the parameter level,
  documenting the phrase's grammatical requirements.
- Multiple parameters can have different governance:
  `give($source:nom, $target:dat, $object:acc) = "{$source} даёт {$target} {$object}"`.

### Real-World Example

The card "Shardwoven Tyrant" has `abandon an ally` (instrumental in Russian) and
`dissolve an enemy with spark less than that ally's spark` (accusative). Without
governance annotations, the translator writes two different case selectors in two
different phrases and must keep them consistent. With governance, the parameter
declaration self-documents which case each verb requires, and omitting the
selector at a use site is safe rather than a bug.

---

## Proposal 4: Transparent Wrapper (`:transparent`)

### Problem

Some phrases exist in the English source to apply English-specific grammar
(articles, pluralization) but are semantic no-ops in many target languages.
`predicate_with_indefinite_article` applies English "a"/"an" — Russian, Chinese,
Japanese, Korean, and Turkish have no indefinite articles.

Currently, the translator must write:

```
predicate_with_indefinite_article($p) = :from($p) "{$p}";
```

This requires understanding `:from` semantics just to express "do nothing."
Worse, if a translator omits `:from`, the phrase silently drops variant and tag
information, causing downstream case selection to fail.

### Proposed Syntax

```
predicate_with_indefinite_article($p) = :transparent($p);
```

**Semantics:** The phrase returns its argument unchanged — same text, same tags,
same variant structure. Equivalent to `:from($p) "{$p}"` but with explicit
intent. No template is needed.

If a phrase has multiple parameters, `:transparent($p)` specifies which parameter
is the identity target. The other parameters are unused (which is already
supported by RLF — unused parameters don't cause errors).

### Justification

- **~10 lines saved** per translation file.
- **Clarity:** `:transparent` communicates "this phrase doesn't apply in my
  language" far more clearly than `:from($p) "{$p}"`.
- **Safety:** A translator who forgets `:from` and writes just `= "{$p}"` will
  silently lose variant information. `:transparent` makes the safe behavior the
  easy behavior.
- Applies to 5-6 phrases in every non-English translation (article application,
  English-specific plural wrappers, vowel-based article selection).

### Real-World Example

The card "Abyssal Plunge" produces `dissolve an enemy with cost 3● or more`. The
predicate "an enemy with cost 3● or more" flows through
`predicate_with_indefinite_article` to add the "an". In Russian, this wrapper
must be transparent — the compound predicate's case variants must pass through
intact so that `dissolve_target` can select accusative on the final result. With
`:transparent`, the translator writes one self-documenting line and the variant
chain is preserved.

---

## Implementation Priority

| Proposal | Impact | Complexity | Languages Affected |
|----------|--------|------------|-------------------|
| 1. `:passthrough` | High | Low-Medium | All case languages (ru, de, tr, ar, fi, hu, pl, ...) |
| 2. `:pattern` | Medium-High | Medium | All inflected languages |
| 3. `:gov` | Medium | Low | All case languages |
| 4. `:transparent` | Low-Medium | Low | All non-English |

Recommended order: 1, 3, 4, 2. Proposals 1, 3, and 4 are small, low-risk macro
expansions. Proposal 2 requires a new parsing construct but is the biggest
quality-of-life improvement for translators working with heavily inflected
languages.

---

## References

- English phrase definitions: `rules_engine/src/strings/src/strings.rs`
- RLF evaluator: `~/rlf/crates/rlf/src/interpreter/evaluator.rs`
- RLF macro codegen: `~/rlf/crates/rlf-macros/src/codegen.rs`
- CLDR plural rules: `~/rlf/crates/rlf/src/interpreter/plural.rs`
- `:from` design: `~/rlf/docs/DESIGN.md` (Section: Metadata Inheritance)
- Variant composition: `~/rlf/docs/PROPOSAL_VARIANT_AWARE_COMPOSITION.md`
- Migration design: `docs/plans/serializer_rlf_migration.md` (Sections 2.6, 4.3)
