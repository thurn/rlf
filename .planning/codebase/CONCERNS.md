# Codebase Concerns

**Analysis Date:** 2026-02-04

## Tech Debt

**Error Handling in Generated Functions:**
- Issue: Generated phrase functions from the `rlf!` macro use `.expect()` and panic on errors, with no graceful degradation
- Files: `docs/APPENDIX_RUST_INTEGRATION.md`, `docs/APPENDIX_DREAMTIDES_ADOPTION.md`
- Impact: Applications using RLF will crash if phrase definitions are missing or malformed at runtime. Static phrases where errors indicate programming mistakes can panic, but dynamic content from external data files should handle errors gracefully
- Fix approach: Keep panic behavior for compile-time validated phrases, but provide better error handling documentation. For data-driven content, always use interpreter methods that return `Result` instead of generated functions

**No Language Fallback to English:**
- Issue: If a phrase exists in English but not in the requested language, RLF returns `PhraseNotFound` error instead of falling back to English
- Files: `docs/DESIGN.md` (line 638-640), `docs/APPENDIX_RUNTIME_INTERPRETER.md`
- Impact: Incomplete translations will break at runtime. Requires complete translation coverage for all languages, increasing translation burden and maintenance complexity
- Fix approach: Document this as a hard requirement in migration guides. Consider adding optional fallback mechanism for production safety, or implement validation tooling to catch missing translations before deployment

**Context-Dependent String Output (Dreamtides Case):**
- Issue: The current system tracks `StringContext` (Interface vs CardText) affecting output formatting, but RLF doesn't have native support for context-aware variant selection
- Files: `docs/APPENDIX_DREAMTIDES_ADOPTION.md` (lines 483-497)
- Impact: When migrating existing localization systems, context requirements may force workarounds (multiple phrase definitions or parameter passing)
- Fix approach: Add optional context parameter support to phrases or document context parameter pattern for applications that need it

---

## Known Limitations

**No Phonetic Guessing for Article Transforms:**
- Issue: The `@a` transform (indefinite article) requires explicit `:a` or `:an` tags; missing tags produce runtime errors instead of phonetic guessing
- Files: `docs/DESIGN.md` (lines 405-406), `docs/APPENDIX_STDLIB.md`
- Impact: Translators and developers must explicitly tag every noun. Forgetting a tag causes runtime failures. More work upfront, but prevents silent incorrect behavior
- Workaround: Use validation tooling to detect missing article tags during development

**StringId Hash Collisions:**
- Issue: `PhraseId` uses hash-based 8-byte identifiers for serializable phrase references
- Files: `docs/APPENDIX_RUST_INTEGRATION.md` (lines 688-706)
- Impact: Theoretical risk of hash collision across large phrase sets. Current implementation doesn't detect or handle collisions
- Fix approach: Document safe limits (likely very high with 8-byte hashes), or add collision detection during phrase registration

---

## Fragile Areas

**Cyclic Reference Detection:**
- Files: `docs/DESIGN.md` (line 609), `docs/APPENDIX_RUST_INTEGRATION.md`
- Why fragile: Compile-time cyclic reference detection is implemented but not heavily tested. Complex multi-phrase templates with mutual references could expose edge cases
- Safe modification: Add comprehensive test cases for cyclic reference patterns before adding features that increase reference complexity
- Test coverage: Need explicit test cases for: direct cycles (A→B→A), multi-level cycles, cycles through phrase calls

**Metadata Inheritance with `:from(param)`:**
- Files: `docs/DESIGN.md` (lines 261-307), `docs/APPENDIX_DREAMTIDES_ADOPTION.md` (lines 209-300)
- Why fragile: The `:from(param)` modifier is powerful but complex—it inherits tags and variants from parameters. Incorrect usage can silently lose grammatical information
- Safe modification: Always ensure callers pass `Phrase` values (not string keys). Add lint warnings when `String` is passed to functions expecting `Phrase`
- Test coverage: Need tests for: missing parameter tags, mismatched parameter/template types, empty variant sets

**Markup in Localized Strings:**
- Files: `docs/APPENDIX_DREAMTIDES_ADOPTION.md` (lines 469-481)
- Why fragile: HTML-like markup (`<color=#00838F>●</color>`, `<b>text</b>`) is embedded directly in phrase content. Translators can accidentally break markup structure
- Safe modification: Add validation tooling to check markup is preserved and balanced in translations
- Test coverage: Need tests for: markup preservation, nested markup, escaped markup characters

**Multi-Dimensional Variant Resolution with Fallbacks:**
- Files: `docs/DESIGN.md` (lines 113-124)
- Why fragile: Variant resolution uses progressively shorter keys (`nom.many` → `nom` → fallback). If fallback chain is incomplete, runtime errors occur
- Safe modification: Add compile-time validation for translation files to ensure complete fallback chains. Document the resolution order clearly
- Test coverage: Need tests for: missing intermediate fallbacks, wildcard ambiguity, empty fallback chains

---

## Missing Critical Features

**No Built-in Plural Rule Validation for New Languages:**
- Problem: Adding a new language requires understanding CLDR plural categories and implementing correct plural rules for that language
- Blocks: Extending localization to new language families (e.g., Chinese, Japanese with measure words) requires custom implementation
- Impact: Medium - adds complexity to internationalization, but documented in appendix

**No Translation Coverage Tooling:**
- Problem: RLF can detect syntax errors in translation files at load time, but provides no tooling to check coverage (which phrases are translated, which are missing)
- Blocks: Large-scale translation projects can't track which phrases need translation before release
- Workaround: Recommended to build custom tooling or use command-line utilities mentioned in docs (`rlf coverage` command)

**No IDE Integration Package:**
- Problem: RLF achieves IDE autocomplete through Rust code generation, but has no plugins for popular editors
- Blocks: Translation file editing in `.rlf` files provides no autocomplete or validation
- Workaround: Manage translations via interpreted content or use command-line validation

**No Serialization Framework for Complex Effects:**
- Problem: The Dreamtides case study (lines 127-155) shows that dynamic card text requires Option A (templates with `eval_str`) or Option B (structured data with phrase calls). Option B is recommended but requires defining phrases for every effect pattern
- Blocks: Data-driven localization becomes verbose without a serialization framework
- Impact: Dreamtides migration path includes this as Phase 4 optimization

---

## Test Coverage Gaps

**Interpreter Runtime Path (High Priority):**
- What's not tested: Full coverage of interpreter evaluation with all primitive combinations, edge cases in parameter passing, transform chaining, and error conditions
- Files: `docs/APPENDIX_RUNTIME_INTERPRETER.md` (Architecture described, but implementation not analyzed)
- Risk: Runtime template evaluation (used in data-driven content) could have subtle bugs if not exhaustively tested
- Priority: High - directly affects production data-driven templates

**Macro Compilation Pipeline (High Priority):**
- What's not tested: Edge cases in macro parsing and code generation - especially error reporting accuracy, span preservation, and edge cases in reference resolution
- Files: `docs/APPENDIX_RUST_INTEGRATION.md` (macro architecture described)
- Risk: Compile errors could have incorrect locations or misleading messages
- Priority: High - affects developer experience

**Translation File Format Validation (Medium Priority):**
- What's not tested: Comprehensive validation of `.rlf` file syntax at load time - error recovery, line number accuracy, multi-dimensional variant parsing
- Files: `docs/APPENDIX_RUNTIME_INTERPRETER.md` (parser described)
- Risk: Translation files with subtle syntax errors could produce unhelpful error messages
- Priority: Medium - affects translator experience

**Phrase Composition with Metadata (Medium Priority):**
- What's not tested: Complex scenarios combining `:from(param)`, multi-key variants, wildcard fallbacks, and transform chaining
- Files: `docs/DESIGN.md` (all features documented)
- Risk: Uncommon combinations could expose bugs in variant resolution or metadata inheritance
- Priority: Medium - affects advanced use cases

---

## Performance Considerations

**Interpreter Performance Not Documented:**
- Issue: Performance characteristics of the runtime interpreter are not discussed
- Files: `docs/APPENDIX_RUNTIME_INTERPRETER.md`
- Impact: Unknown - could be issue for high-frequency phrase evaluation or large translation files
- Concern: Cache warming, phrase lookup performance, template parsing speed not characterized

**Phrase Registry Lookup Strategy:**
- Issue: Implementation details of phrase registry (likely HashMap) not specified
- Files: `docs/APPENDIX_RUNTIME_INTERPRETER.md` (lines 44-46)
- Impact: Hash-based lookup is O(1) but collisions and rehashing could affect performance
- Concern: No mention of phrase count limits or performance degradation curves

---

## Security Considerations

**No Input Validation for Template Parameters:**
- Risk: Parameters are accepted as `Value` enum (numbers, strings, phrases). No validation that parameter types match what transforms/selectors expect
- Files: `docs/DESIGN.md` (lines 550-569), `docs/APPENDIX_RUST_INTEGRATION.md`
- Current mitigation: Type mismatch produces runtime errors, caught at evaluation time
- Recommendations: Consider compile-time type checking for generated functions; document parameter validation for interpreter methods

**No Markup Injection Prevention:**
- Risk: Markup is embedded directly in phrase content. If parameters contain user-generated content with markup characters, output could be malformed
- Files: `docs/APPENDIX_DREAMTIDES_ADOPTION.md` (lines 469-481)
- Current mitigation: None - markup is treated as literal text
- Recommendations: Document safe practices for including user content (escape or validate markup), provide escaping utilities if needed

**Hash-Based PhraseId Not Collision-Resistant:**
- Risk: Using 8-byte hash for serialization creates theoretical collision risk
- Files: `docs/APPENDIX_RUST_INTEGRATION.md` (lines 688-706)
- Current mitigation: Very low probability with 64-bit hash space
- Recommendations: Add collision detection at phrase registration time; log warnings if collisions occur

---

## Dependencies at Risk

**Procedural Macro Stability:**
- Risk: RLF relies on procedural macros for compile-time code generation. Future Rust macro system changes could impact functionality
- Impact: Low - macros are stable, but future breaking changes in macro_rules or proc-macro crate could require updates
- Migration plan: Monitor Rust RFC discussions for macro system changes; maintain compatibility layer if needed

---

## Design Decisions Worth Reconsidering

**Panic-Based Error Handling for Generated Functions:**
- Current: Generated functions use `.expect()` and panic on missing phrases
- Rationale (from docs): "Panics indicate programming mistakes"
- Risk: If phrase definitions are loaded from data files (even if data files are tracked in git), late-loaded translations can panic
- Recommendation: Consider adding configuration option for error handling strategy (panic vs graceful degradation)

**No Language Fallback:**
- Current: Missing translations return error immediately
- Rationale (from docs): "Translations must be complete"
- Risk: Release with incomplete translation causes downtime if English phrases aren't covered
- Recommendation: Add optional fallback mode for production (English fallback with warning) while maintaining strict mode for development

**Strict Tag Requirements:**
- Current: Transforms like `@a` require explicit `:a` or `:an` tags; missing tags cause runtime errors
- Rationale: "No phonetic guessing"
- Risk: Developers must remember all required tags; forgetting a tag breaks production
- Recommendation: Consider adding lints or clippy-style warnings for common tag omissions

---

*Concerns audit: 2026-02-04*
