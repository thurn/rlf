# Phase 10: CLI Tools - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Command-line tools for RLF file validation, template evaluation, and translation coverage checking. Three subcommands: `rlf check`, `rlf eval`, `rlf coverage`. This phase delivers the CLI binary, not library APIs (those exist in earlier phases).

</domain>

<decisions>
## Implementation Decisions

### Output formatting
- Auto-detect: colors if TTY, plain text if piped/redirected
- `--json` flag for machine-readable structured output
- Brief confirmation on success (e.g., "Checked 5 files, no errors")
- `-v` / `--verbose` flag for detailed output (files processed, timing)
- `--no-color` flag to force plain text even on TTY

### Error presentation
- Location format: `file:line:column` (IDE-compatible)
- Show source context with caret pointing to error location
- Include "did you mean" suggestions for typos
- Limit errors to ~10 per file, show "and X more..." if exceeded

### Command structure
- Git-style subcommands: `rlf check`, `rlf eval`, `rlf coverage`
- Global `--help` lists commands, per-command `--help` shows options
- Multiple file arguments supported for `rlf check` (shell handles glob expansion)
- Parameter passing for `rlf eval`: repeated `-p name=value` flags
- Differentiated exit codes: 1 = syntax error, 2 = missing file, etc.

### Coverage report
- ASCII table format for terminal display
- Summary table + detailed missing phrase list per language
- Absolute counts: "17/20 phrases" (not percentages)
- `--strict` flag makes incomplete translations a non-zero exit (for CI)
- Without `--strict`, coverage is informational (exit 0)

### Claude's Discretion
- CLI framework choice (clap, argh, etc.)
- Exact color scheme and styling
- Specific exit code values for each error type
- Verbose output detail level
- Error limit number (approximately 10)

</decisions>

<specifics>
## Specific Ideas

- Exit codes should be useful for CI pipelines (differentiated by error type)
- The `file:line:column` format should work with standard editor integrations

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-cli-tools*
*Context gathered: 2026-02-05*
