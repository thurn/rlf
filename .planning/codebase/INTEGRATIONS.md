# External Integrations

**Analysis Date:** 2026-02-04

## APIs & External Services

**Internationalization Services:**
- ICU4X (Unicode International Components) - Plural rule evaluation and locale data
  - Client: `icu_plurals` crate v2, `icu_locale_core` crate v2
  - Purpose: CLDR-compliant plural category determination for all supported languages
  - Data: Embedded plural rules database (included with crate, no external API calls)

## Data Storage

**Databases:**
- None - RLF is a localization framework, not a data persistence system

**File Storage:**
- Local filesystem only
- Translation files (`.rlf`) loaded from file paths relative to application working directory or absolute paths
- Loading API: `Locale::load_translations(language_code, file_path_string)`
- Format: RLF DSL text files (see `docs/DESIGN.md` for syntax)

**Caching:**
- None documented - Phrases loaded into memory via `RlfInterpreter` at startup
- Typical translation file size: few hundred KB per language

## Authentication & Identity

**Auth Provider:**
- None - RLF is a text rendering library with no authentication requirements

## Monitoring & Observability

**Error Tracking:**
- None - RLF returns `Result` types or panics on errors per context
- Generated phrase functions use `.expect()` and panic on missing phrases (compile-time error indicates programming bug)
- Interpreter methods return `EvalError` results for data-driven templates

**Logs:**
- No logging framework documented - Errors are returned to caller
- Load errors include file path and line number in error messages

## CI/CD & Deployment

**Hosting:**
- Not applicable - RLF is a Rust library, deployed as part of host application

**CI Pipeline:**
- None specified - Deployment determined by host application

## Environment Configuration

**Required Env Vars:**
- None - Configuration via Rust code and file paths

**Secrets Location:**
- Not applicable - No external secrets management

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Plural Rule Data Source

**CLDR (Common Locale Data Repository):**
- Data: Plural rules for all world languages
- Source: Unicode CLDR dataset
- Implementation: Compiled into `icu_plurals` crate at dependency build time
- Access: Via `PluralRules::try_new(locale, PluralRuleType::Cardinal)` API
- No runtime HTTP requests - data is static and embedded

## Procedural Macro Ecosystem

**Rust Toolchain Integration:**
- `syn` (implicit via proc-macro usage) - Token parsing for macro implementation
- `quote` (implicit via proc-macro usage) - Token generation for generated code
- `proc-macro` crate feature - Enabled for `rlf!` macro implementation

## Translation File Loading Pattern

**Runtime Loading:**
```rust
let mut locale = Locale::new();
strings::register_source_phrases(locale.interpreter_mut());
locale.load_translations("ru", "assets/localization/ru.rlf")?;
locale.load_translations("es", "assets/localization/es.rlf")?;
```

**Data-Driven Templates:**
- Interpreter supports runtime template evaluation from TOML, JSON, or other data files
- Templates use identical RLF syntax as phrase definitions
- API: `locale.interpreter().eval_str(template, language, params)?`

---

*Integration audit: 2026-02-04*
