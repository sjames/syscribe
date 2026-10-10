---
type: Requirement
id: REQ-TRS-SETGEN-001
name: "set edits any schema-known scalar or list field with validate-before-write"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
---

`set` shall edit more than `status`, `evidence` and `achieves`, with the same guarantee that an invalid edit is refused before anything is written (GH #242).

## Behavior

- `set <id> <field>=<value>` for a scalar field: `assignedTo`, `responsibility`, `breakdownAdr`, `asilLevel`, `reqDomain`, `reqClass`, `requirementKind`, `verificationMethod`, `testLevel`, `itemType`.
- `set <id> <field>.add <value>` for a list field: `tags`, `blockedBy`, `derivedFrom`, `verifies`, `satisfies`, `confirms`, `hazardousEvents`.
- The edit is applied to a candidate copy of the model and validated. It is refused, with nothing written, when it introduces a new error finding on the edited file (an out-of-enum `asilLevel`, a dangling `blockedBy`, …). New warnings are reported as notes.
- The edit is a line-level splice: every other byte of the file, comments included, is unchanged. `--dry-run` prints the diff and writes nothing.
- An unknown field is refused and the message lists the supported fields. `status`, `evidence.add` and `achieves.add` behave as before.
