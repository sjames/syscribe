---
type: Requirement
id: REQ-TRS-MCP-047
name: "The reference-impact guard and referential-integrity gate resolve references nested inside Allocation features: entries"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-020]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
  - allocation
---

The cross-reference scan that backs both `delete_element`'s reference-impact guard
(`REQ-TRS-MCP-020`) and the referential-integrity commit gate (`REQ-TRS-MCP-008`) shall also
resolve `allocatedFrom`/`allocatedTo` references declared inside a `type: Allocation` element's
`features:` list entries, not only top-level `allocatedFrom`/`allocatedTo` frontmatter fields.

## Rationale

`Allocation` elements in this model conventionally group many allocation pairs as `features:`
list entries, each carrying its own `allocatedFrom`/`allocatedTo` (see
`model/Allocations/RequirementAllocation.md`), rather than as a single top-level scalar/list
field. The shared reference scan (`element_ref_strings`, `crates/syscribe-model/src/mutate/
guard.rs`) previously inspected only top-level fields, so:

- `delete_element` could delete an element referenced by dozens of nested allocation entries
  with no warning and no `blockedBy` entry, silently orphaning those entries — the exact guard
  `REQ-TRS-MCP-020` requires did not fire for this reference shape.
- The same gap meant a create/update that pointed a nested `allocatedFrom`/`allocatedTo` at a
  nonexistent element raised no `EREF` finding, undermining `REQ-TRS-MCP-008`'s referential-
  integrity gate for this one nested reference shape.

## Scope

- Every `features:` entry whose own `type` is `Allocation` and that carries `allocatedFrom` and/or
  `allocatedTo` contributes those values to the same reference scan as the top-level fields, using
  the same field labels (`allocatedFrom`/`allocatedTo`) so `EREF` messages and `blockedBy` entries
  read identically regardless of which shape produced them.
- This requirement governs the shared scan only (`syscribe-model::mutate::guard`), so both the
  MCP `delete_element` tool and `syscribe-server`'s `DELETE /api/elements/{qname}` — which share
  this one implementation — are fixed together.
