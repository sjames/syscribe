---
id: REQ-TRS-SET-001
type: Requirement
name: Tool shall provide a schema-aware set status=<value> command
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `set <qname|id> status=<value>` command that mutates only the
`status:` field of an existing element, resolved by qualified name or stable id. When the
target's type has a known `status:` enum (`Requirement`, `TestCase`, `TestPlan`, `ADR`,
`PlanningItem`, `ReviewRecord`), `<value>` **shall** be validated against that enum before
anything is written — an out-of-enum value is refused with the allowed-value list printed, and
no file is touched. A type with no defined `status:` enum **shall** accept any value
unvalidated, matching `validate`'s own posture for those types.

The write **shall** be a true single-line splice — replacing an existing `status:` line in
place (preserving field order), or appending one if absent — never a full YAML-mapping
round-trip, so every other byte of the file (including another field's quoting style) is
preserved exactly.

When the target is a `PlanningItem` and `<value>` is `done`, the command **shall** additionally
run the `W310` check (`REQ-TRS-PLANITEM-010`) against an in-memory candidate and print any
resulting finding as a non-blocking warning — the write **shall** still proceed regardless,
matching `W310`'s own warning severity.

A `--dry-run` option **shall** print a unified diff of the would-be change without writing
anything.

**Source:** GitHub issue #112.

**Acceptance criteria:**
- `set <id> status=<invalid-enum-value>` is rejected with the allowed value list; no file
  written.
- `set <id> status=<valid-value>` writes only that line — the rest of the file, including an
  unrelated field's quoting, is byte-identical.
- Resolves the target by both qualified name and stable id.
- `set <PlanningItem-id> status=done` whose `achieves:` requirement doesn't meet the `W310` bar
  prints a warning but still commits the write (exit 0).
- `--dry-run` prints the diff and writes nothing.
