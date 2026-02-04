---
phase: quick-001
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - justfile
  - crates/rlf/src/parser/template.rs
  - crates/rlf/src/parser/file.rs
autonomous: true

must_haves:
  truths:
    - "`just no-inline-tests` passes (no #[test] in src/)"
    - "`just test` passes (all tests run)"
    - "Inline tests removed from template.rs and file.rs"
  artifacts:
    - path: "crates/rlf/src/parser/template.rs"
      provides: "Template parser without inline tests"
      contains: "pub fn parse_template"
    - path: "crates/rlf/src/parser/file.rs"
      provides: "File parser without inline tests"
      contains: "pub fn parse_file"
    - path: "justfile"
      provides: "Fixed no-inline-tests check"
      contains: "crates/*/src/"
---

<objective>
Fix the `no-inline-tests` check and remove inline tests from src/ directory.

Purpose: The `just no-inline-tests` check is broken (looks for `src/` at project root, but code is in `crates/rlf/src/`). Additionally, there are 42+ inline tests in `template.rs` and `file.rs` that should be removed since equivalent integration tests already exist in `crates/rlf/tests/`.

Output: A working `no-inline-tests` check and clean src/ with no `#[cfg(test)]` modules.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Current state:
- `just no-inline-tests` incorrectly passes (grep looks for `src/` but code is in `crates/rlf/src/`)
- `crates/rlf/src/parser/template.rs` has `#[cfg(test)] mod tests` at line 257 (42 tests)
- `crates/rlf/src/parser/file.rs` has `#[cfg(test)] mod tests` at line 415
- Integration tests already exist in `crates/rlf/tests/` (template_parser.rs, file_parser.rs)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix justfile no-inline-tests check</name>
  <files>justfile</files>
  <action>
Update the `no-inline-tests` recipe to search `crates/*/src/` instead of `src/`:

```just
no-inline-tests:
    #!/usr/bin/env bash
    if grep -r '#\[test\]' crates/*/src/ 2>/dev/null; then
        echo "Error: #[test] found in src/ directories"
        exit 1
    else
        echo "No inline tests"
    fi
```

This matches the workspace structure where all crates are in `crates/`.
  </action>
  <verify>Run `grep -r '#\[test\]' crates/*/src/` and confirm it finds the existing inline tests (proving the pattern works)</verify>
  <done>The `no-inline-tests` recipe searches `crates/*/src/` for `#[test]` attributes</done>
</task>

<task type="auto">
  <name>Task 2: Remove inline tests from template.rs and file.rs</name>
  <files>crates/rlf/src/parser/template.rs, crates/rlf/src/parser/file.rs</files>
  <action>
Remove the `#[cfg(test)] mod tests { ... }` blocks from both files:

1. In `template.rs`: Delete lines 257-544 (the entire `#[cfg(test)] mod tests` block)
2. In `file.rs`: Delete the `#[cfg(test)] mod tests` block starting at line 415 to end of file

These tests are duplicates of the integration tests in `crates/rlf/tests/` which test the same public API (`parse_template`, `parse_file`).
  </action>
  <verify>Run `grep -r '#\[cfg(test)\]' crates/rlf/src/` and confirm no output</verify>
  <done>No `#[cfg(test)]` modules exist in `crates/rlf/src/`</done>
</task>

<task type="auto">
  <name>Task 3: Verify all checks pass</name>
  <files></files>
  <action>
Run the full review suite to ensure:
1. `just no-inline-tests` now correctly passes (no inline tests)
2. `just test` passes (integration tests still work)
3. `just review` passes (all checks green)
  </action>
  <verify>`just review` passes</verify>
  <done>`just no-inline-tests` passes, `just test` passes with all 79+ integration tests, `just review` is green</done>
</task>

</tasks>

<verification>
- `grep -r '#\[test\]' crates/*/src/` returns no output
- `grep -r '#\[cfg(test)\]' crates/*/src/` returns no output
- `just no-inline-tests` outputs "No inline tests"
- `just test` passes (integration tests in `crates/rlf/tests/` still run)
- `just review` passes
</verification>

<success_criteria>
- The `no-inline-tests` check correctly searches workspace crates
- All inline test modules removed from src/ directories
- All integration tests still pass
- Full review suite passes
</success_criteria>

<output>
After completion, create `.planning/quick/001-move-tests-to-separate-crate-with-public/001-SUMMARY.md`
</output>
