---
phase: 09
plan: 01
subsystem: transforms
tags: [chinese, japanese, korean, cjk, classifiers, counters]
depends_on:
  requires: [08-02]
  provides: [ChineseCount, JapaneseCount, KoreanCount transforms, hangeul dependency]
  affects: [09-02, 09-03]
tech_stack:
  added: [hangeul 0.4]
  patterns: [classifier-tag-lookup]
key_files:
  created: []
  modified:
    - crates/rlf/Cargo.toml
    - crates/rlf/src/interpreter/transforms.rs
    - crates/rlf/tests/interpreter_transforms.rs
decisions:
  - "CJK @count format: {count}{classifier}{text} with no spaces"
  - "Classifier lookup via tag-to-character array with find_classifier helper"
  - "context_to_count defaults to 1 when no context provided"
metrics:
  duration: 3 min
  completed: 2026-02-05
---

# Phase 9 Plan 1: CJK Count Transforms Summary

**One-liner:** Chinese/Japanese/Korean @count transforms with classifier/counter tag systems producing "{count}{classifier}{noun}" format.

## What Was Built

Three language-specific @count transforms for CJK languages with mandatory classifier/counter systems:

1. **ChineseCount** - Chinese measure word classifiers
   - 7 classifiers: zhang (张), ge (个), ming (名), wei (位), tiao (条), ben (本), zhi (只)
   - Output format: "3张牌" (3 + classifier + noun)

2. **JapaneseCount** - Japanese counters
   - 6 counters: mai (枚), nin (人), hiki (匹), hon (本), ko (個), satsu (冊)
   - Output format: "3枚カード" (3 + counter + noun)

3. **KoreanCount** - Korean counters
   - 5 counters: jang (장), myeong (명), mari (마리), gae (개), gwon (권)
   - Output format: "3장카드" (3 + counter + noun)

Also added `hangeul = "0.4"` dependency for Korean @particle transform in Plan 03.

## Key Implementation Details

### Classifier Lookup Pattern
```rust
const CHINESE_CLASSIFIERS: &[(&str, &str)] = &[
    ("zhang", "张"), // Flat objects (cards, paper)
    ("ge", "个"),    // General classifier
    // ...
];

fn find_classifier<'a>(value: &Value, classifiers: &'a [(&str, &str)]) -> Option<&'a str> {
    for (tag, classifier) in classifiers {
        if value.has_tag(tag) {
            return Some(classifier);
        }
    }
    None
}
```

### Context-to-Count Extraction
```rust
fn context_to_count(context: Option<&Value>) -> i64 {
    match context {
        Some(Value::Number(n)) => *n,
        Some(Value::String(s)) => s.parse().unwrap_or(1),
        _ => 1,
    }
}
```

### Transform Registration
- `("zh", "count")` => ChineseCount
- `("ja", "count")` => JapaneseCount
- `("ko", "count")` => KoreanCount

## Commits

| Hash | Type | Description |
|------|------|-------------|
| b2fbf40 | feat | Add hangeul dependency and CJK classifier constants |
| 6c24e23 | test | Add comprehensive tests for CJK @count transforms |

## Test Coverage

Added 24 new tests:
- Chinese @count: 7 tests (zhang, ge, ming, wei, ben, missing tag, default count)
- Japanese @count: 6 tests (mai, nin, hiki, hon, satsu, missing tag)
- Korean @count: 6 tests (jang, myeong, mari, gae, gwon, missing tag)
- Registry tests: 2 tests (lookup, isolation from other languages)
- Edge cases: 3 tests (string context parsing for zh/ja/ko)

Total tests now: 472 passing

## Deviations from Plan

None - plan executed exactly as written. Task 2 (implement transform functions) was combined with Task 1 since adding the TransformKind variants required implementing the functions to satisfy the match arm exhaustiveness check.

## Next Phase Readiness

Ready for Phase 9-02 (Vietnamese and Thai @count transforms). The pattern established here (classifier array + find_classifier helper) can be reused directly.

Ready for Phase 9-03 (Korean @particle transform). The hangeul dependency is now available.
