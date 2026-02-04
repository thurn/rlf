# Codebase Structure

**Analysis Date:** 2026-02-04

## Directory Layout

```
rlf/
├── .planning/              # GSD planning and documentation
│   └── codebase/          # Codebase analysis documents
├── docs/                   # Design and specification documents
│   ├── DESIGN.md          # Core RLF language and semantics
│   ├── APPENDIX_RUNTIME_INTERPRETER.md    # Runtime interpreter design
│   ├── APPENDIX_RUST_INTEGRATION.md       # Macro implementation details
│   ├── APPENDIX_STDLIB.md                 # Standard library transforms
│   ├── APPENDIX_DREAMTIDES_ADOPTION.md    # Case study
│   ├── APPENDIX_RUSSIAN_TRANSLATION.md    # Russian grammar reference
│   ├── APPENDIX_SPANISH_TRANSLATION.md    # Spanish grammar reference
│   └── APPENDIX_RUST_INTEGRATION.md       # Rust integration patterns
├── .git/                   # Git repository
├── .gitignore             # Git ignore rules
├── README.md              # Project overview
└── LICENSE                # Apache 2.0 or equivalent
```

## Directory Purposes

**`.planning/codebase/`:**
- Purpose: GSD codebase analysis documents (generated)
- Contains: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, CONCERNS.md (as applicable)
- Key files: None (this directory is auto-populated by GSD tools)

**`docs/`:**
- Purpose: Complete design specification and reference documentation for RLF
- Contains: Design documentation, API reference, appendices, grammar guides
- Key files:
  - `DESIGN.md`: Complete language specification including primitives, transforms, metadata, file structure
  - `APPENDIX_RUNTIME_INTERPRETER.md`: Interpreter architecture, evaluation algorithm, public API
  - `APPENDIX_RUST_INTEGRATION.md`: Macro phases (parsing, validation, code generation), PhraseId design, generated code examples

## Key File Locations

**Entry Points:**

No executable entry points in this repository. This is a **specification and design repository** for the Rust Localization Framework. The actual implementation would be in separate repositories for:
- `rlf` crate: Runtime interpreter (parser, evaluator, registries, transforms)
- `rlf_macro` crate: Procedural macro (`rlf!`)
- `rlf_cli` crate: Command-line tools (`rlf check`, `rlf eval`, `rlf coverage`)

**Design Documents:**
- `docs/DESIGN.md`: Primary specification (813 lines)
  - Sections: Overview, Primitives (phrase, parameter, variant, selection), Metadata, Transforms, File Structure, Locale Object, Generated API, Compile-time/Runtime Errors, Design Philosophy, Translation Workflow

- `docs/APPENDIX_RUNTIME_INTERPRETER.md`: Interpreter specification (573 lines)
  - Sections: Parser, Evaluator, PhraseRegistry, TransformRegistry, Loading Process, Public API, Error Handling, Performance, CLI Tools

- `docs/APPENDIX_RUST_INTEGRATION.md`: Macro and code generation (1112 lines)
  - Sections: Macro Architecture (3 phases), Validation Checks (7 types), Code Generation, PhraseId Design, Error Reporting, Selection Evaluation, Transform Evaluation, Metadata Inheritance

**Reference Documentation:**
- `docs/APPENDIX_STDLIB.md`: Complete transform library per language
- `docs/APPENDIX_RUSSIAN_TRANSLATION.md`: Russian grammar rules for RLF implementation
- `docs/APPENDIX_SPANISH_TRANSLATION.md`: Spanish grammar rules for RLF implementation
- `docs/APPENDIX_DREAMTIDES_ADOPTION.md`: Real-world adoption case study

## Naming Conventions

**File Naming:**
- Markdown documentation: UPPERCASE.md (e.g., DESIGN.md, ARCHITECTURE.md)
- RLF source files (when in implementation): `*.rlf.rs` for source language (e.g., `strings.rlf.rs`)
- RLF translation files (when in implementation): `{language}.rlf` (e.g., `ru.rlf`, `es.rlf`, `zh_cn.rlf`)

**Directory Naming:**
- Flat structure; no nested module directories in design docs
- Asset directories: `assets/localization/` for translation files
- Generated code: implied `src/localization/` for phrase modules

**Naming Patterns Within RLF:**
- Phrase names: `snake_case` (e.g., `draw`, `card`, `draw_one`)
- Parameters: `snake_case` (e.g., `n`, `amount`, `target`)
- Variant keys: `snake_case` or with dots for multi-dimensional (e.g., `one`, `nom.one`, `acc.many`)
- Metadata tags: flexible, language-specific (e.g., `:a`, `:an`, `:fem`, `:masc`, `:neut`)
- Transform names: `@lowercase` (e.g., `@cap`, `@upper`, `@lower`, `@a`, `@der`, `@el`)

## Where to Add New Code

**For Implementation of RLF Crates (not in this repository):**

**New phrase definitions (source language):**
- File: `src/localization/strings.rlf.rs`
- Pattern: Add to `rlf! { ... }` block
- Example:
  ```rust
  rlf! {
      existing_phrase = "text";
      new_phrase(param) = "text with {param}";
  }
  ```

**New translation phrases:**
- File: `assets/localization/{lang}.rlf`
- Pattern: Add phrase definition with same name as source phrase
- Example (in ru.rlf):
  ```
  new_phrase(param) = "Russian text with {param}";
  ```

**Tests for interpreter:**
- Pattern: Co-locate with implementation in `src/interpreter/*.rs`
- Example: `src/interpreter/evaluator.rs` and `src/interpreter/evaluator_tests.rs`

**CLI tools:**
- Location: `src/bin/check.rs`, `src/bin/eval.rs`, `src/bin/coverage.rs`

**Macro tests:**
- Location: `tests/compile_tests/` for compile-time error tests
- Pattern: Use `trybuild` crate for macro error testing

**Utilities:**
- Shared helpers: `src/lib.rs` or `src/utils.rs`
- Per-module utilities: Inline in module files

## Special Directories

**`.git/`:**
- Purpose: Version control metadata
- Generated: Yes (auto-managed by git)
- Committed: No (excluded from commits)

**`.planning/codebase/`:**
- Purpose: GSD tool outputs (analysis documents)
- Generated: Yes (auto-populated by `/gsd:map-codebase`)
- Committed: Yes (these documents guide future work)

**`docs/`:**
- Purpose: Design specification and reference
- Generated: No (manually authored)
- Committed: Yes (source of truth for RLF design)

## Document Structure Patterns

**Design Documents (`docs/`):**
- Markdown format with clear section hierarchy
- Code examples embedded in `\`\`\`rust ... \`\`\`` blocks
- Tables for comparing alternatives, patterns, or language differences
- Sections numbered for reference: "## Pattern Overview", "## Primitives", etc.
- Cross-references use markdown links (e.g., `See **APPENDIX_STDLIB.md**`)

**Generated Analysis Documents (`.planning/codebase/`):**
- Consistent structure across documents
- Every finding includes file path in backticks (e.g., `src/services/user.ts`)
- Patterns shown with code examples, not just described
- Prescriptive language ("Use X pattern") over descriptive ("X pattern is used")

---

*Structure analysis: 2026-02-04*
