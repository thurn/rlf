# Coding Conventions

**Analysis Date:** 2026-02-04

## Naming Patterns

**Files:**
- Kebab-case for agent files: `gsd-executor.md`, `gsd-plan-checker.md`
- Kebab-case for command files: `new-project.md`, `plan-phase.md`
- UPPERCASE.md for reference/template files: `TESTING.md`, `CONVENTIONS.md`, `PLAN.md`, `SUMMARY.md`
- UPPERCASE for state/artifact files: `PROJECT.md`, `ROADMAP.md`, `REQUIREMENTS.md`, `STATE.md`, `DISCOVERY.md`, `CONTEXT.md`, `RESEARCH.md`, `VERIFICATION.md`, `UAT.md`
- JavaScript hooks in `hooks/` with dash-case: `gsd-check-update.js`, `gsd-statusline.js`

**Functions:**
- camelCase for JavaScript functions: `execSync`, `readFileSync`, `mkdirSync`
- UPPERCASE constants: `cacheDir`, `cacheFile`, `projectVersionFile`
- Variable names describe purpose: `COMMIT_PLANNING_DOCS`, `PLAN_START_TIME`, `PLAN_START_EPOCH`

**Variables:**
- camelCase for local variables: `homeDir`, `cwd`, `installed`, `latest`
- Prefix arrays/collections with plural: `hooks`, `research`, `tasks`
- Suffix mock/test data with suffix: `mockFn`, `testUser`, `testConfig`
- Environment variables in UPPERCASE: `COMMIT_PLANNING_DOCS`

**Types:**
- YAML frontmatter fields use snake_case: `name`, `description`, `tools`, `color`, `allowed-tools`, `depends_on`
- Section headers in Markdown use title case: `## Task Anatomy`, `## Core Workflow`
- Role/objective/context tags in angle brackets: `<role>`, `<objective>`, `<execution_context>`

## Code Style

**Formatting:**
- No configured formatter detected; relies on manual formatting
- Markdown uses standard 2-space indentation for nested lists
- YAML uses 2-space indentation for nesting
- Code blocks use triple backticks with language identifier: ` ```bash`, ` ```typescript`, ` ```json`
- XML-like tags lowercase: `<step>`, `<role>`, `<objective>`, `<process>`

**Linting:**
- No detected linter configuration (no .eslintrc, .prettierrc, biome.json)
- JavaScript code follows Node.js conventions (const/let, arrow functions)
- Markdown adheres to standard structure with frontmatter + content separation

## Import Organization

**Order (JavaScript):**
1. Node.js built-in modules: `require('fs')`, `require('path')`, `require('os')`
2. Third-party modules: `require('child_process')`
3. Local modules/utility functions (rarely used in this codebase)

**Path Aliases:**
- No path aliases detected; uses relative paths for document references
- References to other documents use relative paths: `@PROJECT.md`, `@REQUIREMENTS.md`
- Bash command references use environment variables for dynamic paths

## Error Handling

**Patterns:**
- Silent error catching with empty `catch (e) {}` blocks for non-critical operations: `file check`, `npm version lookup`
- Console output suppressed with `stdio: 'ignore'` for background processes
- Process spawning uses `child.unref()` to allow background process completion
- Fallback values for missing data: `version: '0.0.0'`, `latest: 'unknown'`
- No explicit error logging; processes fail silently if not critical

**Example pattern in `/Users/dthurn/rlf/.claude/hooks/gsd-check-update.js`:**
```javascript
let installed = '0.0.0';
try {
  if (fs.existsSync(projectVersionFile)) {
    installed = fs.readFileSync(projectVersionFile, 'utf8').trim();
  } else if (fs.existsSync(globalVersionFile)) {
    installed = fs.readFileSync(globalVersionFile, 'utf8').trim();
  }
} catch (e) {}
```

## Logging

**Framework:** console (implicit in JavaScript), shell output in bash commands

**Patterns:**
- JavaScript uses stdout via spawn process: `stdio: 'ignore'` for background, default for foreground
- Markdown documentation uses comments in XML tags: `<!-- comment -->`
- Shell commands output results to stdout; errors suppressed with `2>/dev/null` redirect
- Cache files store JSON for status persistence: `gsd-update-check.json` contains structured update check results

## Comments

**When to Comment:**
- Node.js files include header comments explaining purpose: `// Check for GSD updates in background, write result to cache`
- Inline comments explain non-obvious behavior: `// Ensure cache directory exists`, `// Run check in background`
- Comments describe the "why" not the "what": explains design decisions like cache location, fallback versions

**JSDoc/TSDoc:**
- Not used in JavaScript hooks (simple utility scripts)
- Agent/command files have YAML frontmatter descriptions instead of JSDoc
- Markdown files use standard markdown headers and sections for documentation

## Function Design

**Size:**
- Scripts are typically 60-90 lines for complete functionality
- Agent files are 600-1800 lines; broken into named sections with `<step>` tags
- Command files are 300-1000 lines; include execution context + detailed guidance

**Parameters:**
- JavaScript functions use object destructuring for options: `{ recursive: true }`, `{ encoding: 'utf8' }`
- Bash commands use positional arguments: `$1`, `$2` or environment variables: `${VAR_NAME}`
- Markdown sections use structured data in YAML blocks or tables

**Return Values:**
- JavaScript returns JSON structures: `{ update_available: boolean, installed: string, latest: string, checked: number }`
- Bash commands return exit codes; output captured via `$(command)` syntax
- Markdown files are the primary output/return type

## Module Design

**Exports:**
- Node.js files typically don't export; they run standalone via `#!/usr/bin/env node` shebang
- YAML frontmatter defines module identity: `name`, `description`, `tools`, `color`
- Markdown files export as documents; no code modules

**Barrel Files:**
- Not applicable; this codebase doesn't use module bundling
- Agent/command organization via directory structure: `.claude/agents/`, `.claude/commands/gsd/`
- Files are independently accessed by name via frontmatter `name` field

## Structural Patterns

**XML-like Tags:**
Files use semantic XML-style tags for sections:
- `<role>` - defines agent responsibility
- `<objective>` - defines command purpose
- `<process>`, `<step>` - decompose complex flows
- `<execution_flow>`, `<upstream_input>`, `<core_principle>` - sections of agent behavior
- `<verification_dimensions>` - breakdown of verification criteria
- `<good_examples>`, `<guidelines>`, `<critical_rules>` - reference content

**Frontmatter Structure:**
All executable files (agents, commands) use YAML frontmatter:
```yaml
---
name: command-or-agent-name
description: Human-readable description
tools: Tool1, Tool2, Tool3
color: cyan|green|yellow|etc
---
```

**Agent Mandatory Sections (in order):**
1. `<role>` - who the agent is
2. Core logic sections (varies by agent)
3. `<step>` sections with `name` and `priority` attributes
4. Return/completion section

**Reference Files (docs, templates):**
- Start with markdown heading: `# Reference Title`
- Include description of purpose
- Markdown formatting with headers, lists, code blocks
- Include `<guidelines>` and `<good_examples>` sections for templates

---

*Convention analysis: 2026-02-04*
