---
type: ADR
id: ADR-OUT-001
name: "list TestCase: CI-oriented output format and AND-semantics multi-tag filtering"
status: accepted
---

## Context

The `list` command produces a generic element table (`Qualified Name | Name / ID | Supertype | File`)
and a generic JSON envelope (`qualifiedName`, `type`, `name`, `id`, `status`). That format is
useful for model navigation but misses the execution-specific metadata a CI runner needs to:

- select the right tests for a build variant (`--config`)
- narrow to a specific workload (e.g. "all integration tests that are also safety-tagged")
- map test results back to requirements (`verifies`)
- invoke the actual test functions (`sourceFile`, `testFunctions`)

A naive `--tag` flag with OR semantics (union of tag groups) would only broaden the result set,
forcing CI pipelines to add their own filtering. The qualification suite's `list` is the primary
machine-readable test-selection surface.

## Decision

1. **Specialised table for TestCase** (`REQ-TRS-OUT-014`): when the type filter is `TestCase`,
   emit `ID | Name | Level | Status | Verifies | Tags` instead of the generic schema. The `list`
   dispatch stays a single command; the specialisation is purely in the output stage.

2. **AND semantics for multi-tag `--tag`** (`REQ-TRS-TAG-002`): repeating `--tag` narrows the
   result. This differs from the OR (union) semantics on other tag-aware commands
   (`REQ-TRS-TAG-001`). The distinction is intentional: `list` is the narrowing surface; other
   commands (search, audit) broaden. Changing `list` to AND does not affect other commands.

3. **JSON extension for TestCase**: `testLevel`, `verifies`, `tags`, `sourceFile`,
   `testFunctions` added alongside the base fields. The contract is additive — existing consumers
   of the generic JSON are unaffected by extra keys.

4. **`--config` interaction**: projection is orthogonal and composes with tag filtering.
   Applies-when evaluation runs first; the result feeds the tag filter.

## Alternatives considered

- **Separate `list-tests` sub-command** — rejected: adds a parallel CLI surface with the same
  semantics; `--config` and `--tag` composition would need to be replicated.
- **OR semantics for multi-tag** — rejected: a CI runner wanting "integration AND safety" would
  have to post-filter; AND is strictly more useful as the primitive (OR is recoverable by running
  two invocations; AND is not).
- **Extra columns on all types** — rejected: non-TestCase elements don't carry `testLevel` or
  `testFunctions`; sparse columns are confusing and padding wastes space.

## Consequences

- `list TestCase` is now a specialised view; other types keep the generic table.
- `--tag` behaviour diverges between `list` and other commands — documented in `REQ-TRS-TAG-001`
  / `REQ-TRS-TAG-002` cross-references and in the CLI usage string.
- TC-TRS-OUT-014 and TC-TRS-TAG-002 gate correctness at TCL2.
