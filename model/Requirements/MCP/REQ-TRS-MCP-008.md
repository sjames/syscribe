---
type: Requirement
id: REQ-TRS-MCP-008
name: "Write tools are guarded by dry-run, a validation delta, and a referential-integrity commit gate"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

Every MCP write tool (`create_element`, `update_element`, `move_element`) shall implement a
common write-guard protocol so an LLM can propose a change, inspect its effect, and only then
commit it.

## Guard protocol

- **`dry_run` defaults to true.** A call without an explicit `dry_run: false` shall compute the
  effect of the change in memory and **not** modify the disk. Disk shall be byte-for-byte
  unchanged after any `dry_run` call.
- **Validation delta.** Every write call (dry-run or commit) shall return the change to the
  validation state it would cause — the set of newly introduced and newly resolved findings —
  computed by comparing validation before and after the would-be change, partitioned into
  `newErrors`/`resolvedErrors`/`newWarnings`/`resolvedWarnings`.
- **Referential-integrity commit gate.** A commit (`dry_run: false`) shall be refused, returning
  the delta and `written: false`, if the change would break referential integrity — i.e. leave a
  cross-reference (`supertype`, `typedBy`, `redefines`, `subsets`, `verifies`, `derivedFrom`,
  `satisfies`, `allocatedFrom`/`allocatedTo`) that resolved before the change unresolved after it
  (built-in stdlib types exempt) — reported as `newErrors`, unless an explicit override is
  configured (`SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1`). The gate intentionally targets graph corruption
  rather than every validator `Error`: incremental authoring of incomplete drafts (e.g. a stub
  requirement whose normative body is not yet written, which the full validator flags `E012`)
  must remain creatable. Whole-model validator warnings introduced or resolved by the change are
  surfaced in `newWarnings`/`resolvedWarnings` for context but do not block.
- **Store rebuild.** After a successful commit the in-memory store shall be rebuilt so
  subsequent reads reflect the change.
