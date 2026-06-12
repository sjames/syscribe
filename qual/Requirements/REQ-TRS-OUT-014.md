---
id: REQ-TRS-OUT-014
type: Requirement
name: "list TestCase shall emit a test-execution-oriented table and JSON with testFunctions"
status: draft
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-TRS-OUT-001
breakdownAdr: ADR-OUT-001
---

When the `list` command's type filter is `TestCase`, the tool **shall** produce output
specialized for test-execution workflows rather than the generic element table.

## Human-readable table

The table **shall** contain the columns: `ID | Name | Level | Status | Verifies | Tags`.

- **ID** — the stable `TC-*` identifier.
- **Name** — the `name:` label (free prose).
- **Level** — `testLevel:` value (L1–L5), or `—` if absent.
- **Status** — `status:` value.
- **Verifies** — comma-joined `verifies:` list (requirement IDs), or `—` if empty.
- **Tags** — comma-joined `tags:` list, or `—` if none.

## JSON output (`--json`)

In addition to the base fields already emitted by `list --json` (`qualifiedName`, `type`,
`name`, `id`, `status`), a TestCase element **shall** include:

- `testLevel` — string or `null`.
- `verifies` — array of requirement-ID strings (empty array if none).
- `tags` — array of tag strings (empty array if none).
- `sourceFile` — string path or `null`.
- `testFunctions` — array of objects as declared in frontmatter
  (`[{"function": "...", "scenario": "..."}]`), or empty array if none.

These fields give a CI runner everything it needs to invoke the actual test functions and
trace results back to requirements.

## Interaction with `--config`

Both the table and JSON output respect the configuration-projection lens
([[REQ-TRS-PROJ-001]]): `list TestCase --config CONF-X` returns only TestCases active in
that configuration.

**Acceptance criteria:**

- `list TestCase` produces a table with columns `ID | Name | Level | Status | Verifies | Tags`.
- `list TestCase --json` emits `testLevel`, `verifies`, `tags`, `sourceFile`, and
  `testFunctions` on every item.
- `list TestCase --config CONF-X` returns only the TestCases active in that configuration
  (projected via `appliesWhen`).
- `list TestCase --config CONF-X --tag integration` combines projection and tag filtering.
- `list Requirement` and other types continue to use the generic table format (no regression).
