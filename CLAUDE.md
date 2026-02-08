# PROJECT CONTEXT


RLF (Rust Localization Framework) is a localization DSL embedded in Rust via
macros. It provides compile-time validation for source language strings while
supporting runtime loading of translations. The framework uses two kinds of
definitions -- terms and phrases -- along with variants, parameters, selection,
and transforms to produce localized text.

Key concepts:
- **Term**: A named definition without parameters, representing a lexical entry
  with optional variant forms (e.g., singular/plural). Example:
  `card = :a { one: "card", *other: "cards" };`
- **Phrase**: A named definition with parameters (`$`-prefixed), representing a
  template that produces text. Example:
  `draw($n) = "Draw {$n} {card($n)}.";`
- **Parameter**: Values passed to phrases, prefixed with `$` and interpolated
  with `{}`. Example: `{$n}`, `{$entity}`
- **Variant**: Multiple forms of a term (e.g., singular/plural, grammatical
  case). Selected via `:` for static keys or `()` for dynamic parameter-based
  selection.
- **Selection**: The `:` operator selects a static variant key (e.g.,
  `{card:other}`). Parenthesized syntax selects dynamically based on a parameter
  value (e.g., `{card($n)}`).
- **Transform**: The `@` operator modifies text (e.g., `@cap` for capitalize).
  Static context uses `:` (e.g., `@der:acc`), dynamic context uses `()` (e.g.,
  `@count($n)`).
- **Metadata tags**: Grammatical hints like `:fem`, `:masc`, `:a`, `:an`
- **`:match` keyword**: Enables conditional variant selection based on parameter
  tags. Example: `destroyed = :match($thing) { fem: "destruida", *masc: "destruido" };`
- **`:from` keyword**: Causes a phrase to inherit tags from a parameter.
  Example: `subtype($s) = :from($s) "<b>{$s}</b>";`
- **Default variant marker `*`**: Marks a variant as the fallback when no key
  matches. Example: `{ one: "card", *other: "cards" }`

See `docs/DESIGN.md` for the complete language specification.


# DESIGN DOCUMENTATION


- `docs/DESIGN.md` - Canonical language design and syntax reference
- `docs/APPENDIX_STDLIB.md` - Standard library transforms documentation
- `docs/APPENDIX_RUST_INTEGRATION.md` - Rust API and type definitions
- `docs/APPENDIX_RUNTIME_INTERPRETER.md` - Runtime interpreter API
- `docs/APPENDIX_RUSSIAN_TRANSLATION.md` - Russian translation patterns
- `docs/APPENDIX_SPANISH_TRANSLATION.md` - Spanish translation patterns
- `docs/APPENDIX_DREAMTIDES_ADOPTION.md` - Integration example with Dreamtides


# ACCEPTANCE CRITERIA


Please follow this checklist after completing any task:

1) Add error handling where appropriate. Consider edge cases.
2) Add tests where appropriate. Rust tests live in `crates/*/tests/` and
   always test crate public APIs. Test error cases. When fixing a bug,
   reproduce the bug via a test *before* fixing it.
3) Run `just fmt` to apply formatting rules.
4) Run `just review` to run clippy, validate style, and run unit tests.
5) Create a git commit with a detailed description of your work.


# CODE STYLE


- Prefer writing code inline (when possible) to creating new variables via "let" statements
- Add a short doc comment to top-level public functions, fields, and types. Don't add inline comments.
- DO NOT fully-qualify names in code
- Use modern Rust features such as let-else statements and "{inline:?}" variable formatting
- Do not write inline `mod tests {}` tests, place them in the `/tests/` directory
- Do not write code only used by tests. Test against real public API.


# JUST COMMANDS


- After completing work, please always run "just fmt" to apply rustfmt
  formatting rules
- Please use `just` commands instead of `cargo`, e.g. `just fmt`, `just check`,
  `just clippy`, `just test`
- Run `just check` to type check code
- Run `just clippy` to check for lint warnings
- Run `just test` to run all tests
- After completing work, please ALWAYS run `just review` to validate changes
- Do not print a summary of changes after completing work.
- Prefer the `just` commands over `cargo` commands since they have project-specific rules


# CODE STRUCTURE


The code is structured as a series of Rust crates using the cargo "workspace" feature.

```
rlf/
  justfile
  Cargo.toml           # Workspace configuration
  crates/
    rlf/               # Core library (interpreter, runtime)
    rlf-macros/        # Proc macro crate (rlf! macro)
  docs/                # Design documentation
```

The `rlf` crate contains the core interpreter and runtime types. The `rlf-macros`
crate provides the `rlf!` procedural macro that compiles source language phrases
and generates the Rust API.
