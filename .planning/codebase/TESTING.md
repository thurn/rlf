# Testing Patterns

**Analysis Date:** 2026-02-04

## Test Framework

**Runner:**
- Not detected - no formal test framework used
- Code is primarily documentation/markdown configuration, not executable code
- Hooks are utility scripts without test suites

**Assertion Library:**
- Not applicable - testing would require infrastructure changes

**Run Commands:**
```bash
# No test commands configured
# Manual verification of agent behavior only
```

## Test File Organization

**Location:**
- No test files detected in codebase
- Testing templates exist for reference: `.claude/get-shit-done/templates/codebase/testing.md`
- Agent code is self-documenting markdown with execution guidance

**Naming:**
- Not applicable - no test files present

**Structure:**
```
.claude/
├── agents/                    # Agent specifications (no tests)
├── commands/gsd/              # Command specifications (no tests)
├── get-shit-done/
│   ├── templates/codebase/
│   │   └── testing.md         # Template for test documentation
│   ├── references/            # Reference guides (not tests)
│   └── workflows/             # Workflow specifications (not tests)
└── hooks/                     # Utility scripts (no tests)
```

## Test Structure

**Suite Organization:**
Not applicable - agent definitions are documentation, not executable code requiring tests.

**Patterns:**
- Agents and commands define behavior through markdown specification
- Testing occurs via orchestrator execution (external to this codebase)
- Manual verification through `/gsd:execute-phase` orchestrator

## Mocking

**Framework:**
Not applicable to this codebase.

**Patterns:**
Not applicable - primary code is specification/documentation, not functions to mock.

**What to Mock:**
- JavaScript hooks might mock file system in unit tests (but no tests exist)
- External service calls (e.g., npm version check) could be mocked

**What NOT to Mock:**
- Agent/command specifications are deterministic logic, not externals requiring mocks

## Fixtures and Factories

**Test Data:**
- TESTING.md template at `.claude/get-shit-done/templates/codebase/testing.md` includes factory examples
- Example factory pattern:
```typescript
function createTestUser(overrides?: Partial<User>): User {
  return {
    id: 'test-id',
    name: 'Test User',
    email: 'test@example.com',
    ...overrides
  };
}
```

**Location:**
- Not applicable to this codebase
- Reference: `.claude/get-shit-done/templates/codebase/testing.md` line 115-139

## Coverage

**Requirements:**
- Not enforced - codebase is specification/infrastructure, not application code
- Testing template recommends 80% for application codebases (see `TESTING.md` template)

**View Coverage:**
```bash
# Not applicable - no coverage tools configured
```

## Test Types

**Unit Tests:**
- Not currently used
- Would test individual agent logic if application code existed
- Reference template recommends: test single function in isolation, mock external dependencies

**Integration Tests:**
- Not currently used
- Would test multiple agents working together via orchestrator
- Reference template recommends: mock external services, use real internal modules

**E2E Tests:**
- Not currently used in infrastructure codebase
- End-to-end testing occurs through `/gsd:execute-phase` orchestrator (external)
- User verifies phase goals achieved after execution

## Common Patterns

**Async Testing:**
Reference template at `.claude/get-shit-done/templates/codebase/testing.md` line 176-183:
```typescript
it('should handle async operation', async () => {
  const result = await asyncFunction();
  expect(result).toBe('expected');
});
```

**Error Testing:**
Reference template at `.claude/get-shit-done/templates/codebase/testing.md` line 188-200:
```typescript
it('should throw on invalid input', () => {
  expect(() => functionCall()).toThrow('error message');
});

// Async error
it('should reject on failure', async () => {
  await expect(asyncCall()).rejects.toThrow('error message');
});
```

## Agent Behavior Verification

While formal tests don't exist, agents are verified through:

**Specification Validation:**
- Agent YAML frontmatter includes: `name`, `description`, `tools`, `color`
- File: verify frontmatter is complete for all agents in `.claude/agents/`

**Execution Verification:**
- Agents spawn via orchestrator; verify they follow their defined `<role>` section
- File: `.claude/agents/gsd-executor.md` defines executor responsibilities
- Success: phase completes with SUMMARY.md created, STATE.md updated, per-task commits created

**Reference Testing:**
- Templates at `.claude/get-shit-done/templates/codebase/` define expected patterns for new codebases
- User codebases should have `.planning/codebase/TESTING.md` matching template structure
- Validation: `/gsd:map-codebase quality` produces TESTING.md with actual patterns from scanned codebase

## Key Files for Testing Context

**Testing Guidance:**
- `.claude/get-shit-done/templates/codebase/testing.md` - Template with guidelines and good examples (480 lines)

**Agent Specifications:**
- `.claude/agents/gsd-executor.md` - Execution verification patterns
- `.claude/agents/gsd-verifier.md` - Post-execution verification logic
- `.claude/agents/gsd-plan-checker.md` - Pre-execution plan validation

**Reference Patterns:**
- `.claude/get-shit-done/references/verification-patterns.md` - How verification is performed

---

*Testing analysis: 2026-02-04*
*This codebase is infrastructure (agents/commands) not application code. Application codebases created via GSD should follow patterns in `.claude/get-shit-done/templates/codebase/testing.md`*
