# Plan: Explicit Numeric Variant Selectors

## Background

Currently, when a numeric value is used as a selector, RLF **always** maps it
through CLDR plural categories before looking up a variant:

```
card = { one: "card", other: "cards" };
draw(n) = "Draw {n} {card(n)}.";

// n=1 → plural_category("en", 1) → "one" → variant "one" → "card"
// n=3 → plural_category("en", 3) → "other" → variant "other" → "cards"
```

This works well for grammatical number agreement, but sometimes you want to
match a **specific number** rather than a plural category. For example:

```
card = {
    0: "no cards",
    1: "a card",
    other: "cards",
};
draw(n) = "Draw {card(n)}.";

// n=0 → "Draw no cards."
// n=1 → "Draw a card."
// n=3 → "Draw 3 cards."
```

This is a common pattern in localization (ICU MessageFormat calls it "exact value
matching" with `=0`, `=1`, etc.). It lets you provide natural-sounding text for
specific counts without being constrained to CLDR categories.

## Synergy with Parenthesized Dynamic Values

The parenthesized syntax from `PLAN_PARENTHESIZED_DYNAMIC_VALUES.md` makes
numeric selectors straightforward. With the `:` vs `()` distinction:

| Syntax        | Meaning                                    |
| ------------- | ------------------------------------------ |
| `{card:other}` | Static: look up variant key `"other"`     |
| `{card:3}`    | Static: look up variant key `"3"`          |
| `{card(n)}`   | Dynamic: resolve `n` through CLDR, with numeric fallthrough |

The static form `{card:3}` is unambiguous—it always means "find the variant
keyed by the string `3`." The dynamic form `{card(n)}` resolves via CLDR but
**tries the literal number first** (see Resolution Order below).

Without the parenthesized syntax, `{card:3}` under the old `:` system would be
ambiguous—is `3` a literal key or a numeric value to run through CLDR? The
parenthesized syntax eliminates this entirely.

## Proposed Behavior

### Variant Definition

Allow numeric literals as variant keys:

```
card = {
    0: "no cards",
    1: "a card",
    2: "a pair of cards",
    one: "card",
    other: "cards",
};
```

Numeric variant keys are strings internally (`VariantKey("0")`,
`VariantKey("1")`), just like CLDR category names. They coexist with category
keys in the same variant map.

### Static Numeric Selection

Using `:` with a number always looks up the literal key:

```
zero_cards = "You have {card:0}.";   // → "You have no cards."
one_card = "You have {card:1}.";     // → "You have a card."
pair = "You have {card:2}.";         // → "You have a pair of cards."
```

This is useful for hardcoded references to specific count variants.

### Dynamic Selection Resolution Order

When `{card(n)}` is evaluated with a numeric parameter, the resolution order is:

1. **Exact numeric key**: Try the literal number as a string (e.g., n=0 → key `"0"`)
2. **CLDR plural category**: Map through `plural_category()` (e.g., n=3 → `"other"`)
3. **Fallback chain**: Standard dot-separated fallback for multi-dimensional keys

This means exact numeric matches take priority over CLDR categories:

```
card = {
    0: "no cards",
    1: "a card",
    one: "card",
    other: "cards",
};
draw(n) = "Draw {card(n)}.";

// n=0 → try "0" ✓ → "no cards"      (exact match wins)
// n=1 → try "1" ✓ → "a card"        (exact match wins over CLDR "one")
// n=2 → try "2" ✗ → CLDR "other" ✓  → "cards"
// n=5 → try "5" ✗ → CLDR "other" ✓  → "cards"
```

This matches ICU MessageFormat's resolution order, where `=N` takes precedence
over plural categories.

### Multi-Dimensional Numeric Keys

Numeric keys compose with dot notation:

```
// Russian with explicit overrides
card = {
    0: "нет карт",
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};
draw(n) = "Возьмите {card:acc(n)}.";

// n=0  → try "acc.0" ✗ → try "0" ✓ → "нет карт"  (fallback to shorter key)
// n=1  → try "acc.1" ✗ → CLDR "acc.one" ✓ → "карту"
// n=5  → try "acc.5" ✗ → CLDR "acc.many" ✓ → "карт"
```

If you want a case-specific zero form:

```
card = {
    nom.0: "нет карт",
    acc.0: "без карт",
    nom.one: "карта",
    // ...
};
```

### Interpolation Inside Numeric Variants

Numeric variants can include `{n}` interpolation so the count still appears:

```
card = {
    0: "no cards",
    1: "a card",
    one: "card",
    other: "cards",
};

draw(n) = "Draw {card(n)}.";
draw_n(n) = "Draw {n} {card(n)}.";

// draw(0)   → "Draw no cards."
// draw(3)   → "Draw cards."       (no count — "other" variant has no {n})
// draw_n(3) → "Draw 3 cards."     (count comes from {n} in the template)
```

The count is **not** automatically inserted. If a numeric variant wants to
include the number, the template must interpolate `{n}` separately. This keeps
variant definitions simple and composable.

## Complete Example

```
// English
item = {
    0: "no items",
    1: "an item",
    one: "item",
    other: "items",
};

inventory(n) = "You have {n} {item(n)} remaining.";
// n=0 → "You have 0 no items remaining."  — awkward!

// Better pattern: use the numeric variants in a count-aware phrase
inventory(n) = "You have {item_count(n)} remaining.";
item_count(n) = {
    0: "no items",
    1: "one item",
    other: "{n} items",
};

// n=0 → "You have no items remaining."
// n=1 → "You have one item remaining."
// n=3 → "You have 3 items remaining."
```

The second pattern—putting numeric overrides on the template phrase itself rather
than the noun—is generally more natural since different counts may need entirely
different sentence structures.

## Implementation Plan

### Phase 1: Variant Key Parsing

**Files:**
- `crates/rlf-macros/src/parse.rs`
- `crates/rlf/src/parser/template.rs`
- `crates/rlf/src/parser/phrase.rs` (or wherever variant definitions are parsed)

Allow numeric tokens as variant keys in variant definitions:

```
card = {
    0: "no cards",      // ← numeric key
    1: "a card",        // ← numeric key
    one: "card",        // ← identifier key (existing)
    other: "cards",     // ← identifier key (existing)
};
```

**Macro parser:** Currently uses `syn::Ident` for variant keys, which does not
accept bare numbers. Change to accept either an `Ident` or a `LitInt`, then
convert both to string for `VariantKey`.

**Runtime parser:** The `winnow`-based parser for `.rlf` files needs to accept
digit sequences as variant keys in addition to identifiers.

### Phase 2: Static Numeric Selector Parsing

**Files:**
- `crates/rlf-macros/src/parse.rs`
- `crates/rlf/src/parser/template.rs`

Allow numeric literals after `:` in selector position:

```
{card:0}    // static numeric selector
{card:3}    // static numeric selector
```

Both parsers currently accept identifiers after `:`. Extend to also accept
digit sequences. The parsed result is `Selector::Literal("3")` (using the
type from `PLAN_PARENTHESIZED_DYNAMIC_VALUES.md`).

### Phase 3: Dynamic Resolution with Numeric Priority

**Files:**
- `crates/rlf/src/interpreter/evaluator.rs`

Update `resolve_selector_candidates()` to return **two** candidate lists for
numeric parameters—the exact number first, then the CLDR category:

```rust
Selector::Parameter(name) => {
    let value = ctx.get_param(name)?;
    match value {
        Value::Number(n) => {
            // Try exact number first, then CLDR category
            Ok(vec![n.to_string(), plural_category(lang, *n).to_string()])
        }
        // ... other types unchanged
    }
}
```

Update `variant_lookup()` to try candidates in order, returning the first match:

```rust
fn variant_lookup(
    variants: &HashMap<VariantKey, String>,
    candidate_keys: &[String],   // ordered by priority
) -> Result<String, EvalError> {
    for key in candidate_keys {
        if let Some(text) = variants.get(&VariantKey::new(key)) {
            return Ok(text.clone());
        }
    }
    // Then try fallbacks (dot-stripping) for each candidate...
    Err(EvalError::NoMatchingVariant { ... })
}
```

The key change is that `resolve_selector_candidates` currently returns
candidates as alternatives for a single dimension (e.g., multiple tags from a
phrase). For numeric resolution, we need **ordered priority**: try `"3"` before
`"other"`. The `apply_selectors` / `variant_lookup` pipeline needs to respect
this ordering.

### Phase 4: Multi-Dimensional Numeric Keys

When building compound keys from multiple selector dimensions, numeric priority
must be preserved per-dimension:

```
// Selectors: :acc(n) where n=0
// Dimension 1 candidates: ["acc"]
// Dimension 2 candidates: ["0", "other"]  (ordered)
//
// Try in order:
//   1. "acc.0"     ← exact numeric
//   2. "acc.other" ← CLDR fallback
//   3. "acc"       ← dot-fallback
//   4. "0"         ← dot-fallback of numeric
//   5. "other"     ← dot-fallback of CLDR
```

The `build_compound_keys` function currently produces a flat cartesian product.
It needs to produce an **ordered** list respecting the priority within each
dimension.

### Phase 5: Tests

**Files:**
- `crates/rlf/tests/interpreter_eval.rs`
- `crates/rlf-macros/tests/` (trybuild tests)

New test cases:

| Test | Input | Expected |
|------|-------|----------|
| Numeric variant key 0 | `card = { 0: "none", other: "cards" }; {card(n)}` n=0 | `"none"` |
| Numeric variant key 1 | `card = { 1: "a card", one: "card", other: "cards" }; {card(n)}` n=1 | `"a card"` (exact wins over CLDR "one") |
| Numeric fallthrough | Same as above, n=3 | `"cards"` (no "3" key, falls to CLDR "other") |
| Static numeric selector | `{card:0}` | `"none"` |
| Multi-dim numeric | `card = { acc.0: "без карт", acc.one: "карту" }; {card:acc(n)}` n=0 | `"без карт"` |
| Numeric key not present | `card = { one: "card", other: "cards" }; {card(n)}` n=0 | `"cards"` (no "0", CLDR "other") |
| Negative number | `card = { -1: "deficit", other: "cards" }; {card(n)}` n=-1 | `"deficit"` (if supported) |
| Large number | `{card(n)}` n=1000 | Falls to CLDR "other" |
| Parametric variant phrase | `item_count(n) = { 0: "no items", other: "{n} items" };` n=0 | `"no items"` |
| Russian multi-dim | `card = { 0: "нет карт", nom.one: "карта" }; {card:nom(n)}` n=0 | `"нет карт"` (fallback from "nom.0" to "0") |

### Phase 6: Documentation

**Files:**
- `docs/DESIGN.md` — add numeric variant keys to the Variant section, add resolution order
- `docs/APPENDIX_STDLIB.md` — if any transforms interact with numeric keys
- Other appendices as needed

## Design Decisions

### Why exact-number-first resolution order?

ICU MessageFormat uses the same precedence: exact values (`=0`, `=1`) override
plural categories. This matches user expectations—if you define a specific form
for zero, you want it used when the count is zero, regardless of what CLDR says
the plural category for zero is (which varies by language).

### Why not require a prefix like `=0`?

ICU MessageFormat uses `=0`, `=1` to distinguish exact matches from category
names. We don't need this because:

1. CLDR category names (`zero`, `one`, `two`, `few`, `many`, `other`) are all
   alphabetic strings. Numeric keys (`0`, `1`, `2`) are digit strings. They're
   syntactically disjoint—no ambiguity.

2. Adding `=` prefix adds visual noise for a common pattern.

3. If someone names a variant `0` they clearly mean the number zero, not a CLDR
   category (there is no CLDR category `0`).

### Should negative numbers be supported?

Negative numbers as variant keys (e.g., `-1: "deficit"`) add parsing complexity
because `-` is not typically part of an identifier. **Recommendation: defer
negative number support.** If needed later, it can be added as a separate
feature. Negative counts are rare in localization.

### Should float keys be supported?

**No.** Float keys (`1.5: "one and a half"`) create ambiguity with
multi-dimensional dot notation (`nom.one` would look like a float). Floats are
truncated to integers for CLDR resolution, and exact float matching is rarely
useful. Keep keys as integers or identifiers only.

### What about the `@plural` transform interaction?

The `@plural` transform selects the `"other"` variant from a phrase. It does not
interact with numeric keys since it uses a literal category name. This is
correct behavior—`@plural` means "give me the plural form", not "give me the
form for some specific count."

### Impact on `VariantKey` type

`VariantKey` is already a `String` newtype. Numeric keys are stored as their
string representation (`"0"`, `"1"`, `"42"`). No type changes needed.

## Migration

This is a **purely additive** change. No existing syntax is altered. Phrases
that don't use numeric variant keys continue to work identically. The only
behavioral change is in the dynamic resolution order for `{phrase(n)}`:

- **Before**: n=0 → `plural_category("en", 0) → "other"` → looks up `"other"`
- **After**: n=0 → try `"0"` first → if not found, `plural_category("en", 0) → "other"` → looks up `"other"`

Since no existing variant maps contain numeric keys, the "try `"0"` first" step
will always miss, falling through to the existing CLDR behavior. Existing code
is unaffected.

## Dependency on Parenthesized Syntax

This feature is **independent** of but **enhanced by** the parenthesized syntax
change. Without parenthesized syntax:

- `{card:3}` is ambiguous: is `3` a literal key or a number to run through CLDR?
- The evaluator must guess based on whether `3` is a parameter name (it isn't)
  or a literal (it is)—which happens to work but is fragile.

With parenthesized syntax:

- `{card:3}` unambiguously means "look up variant key `3`" (static)
- `{card(n)}` where n=3 means "resolve dynamically" (try `"3"` then CLDR)

**Recommendation:** Implement the parenthesized syntax change first, then add
numeric selectors. The combined result is cleaner and less ambiguous.
