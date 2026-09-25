---
type: Requirement
id: REQ-TRS-SYSMLV2-029
name: "A SysMLv2 allocation def maps to the native AllocationDef, so an ingested allocation usage's typedBy: resolves and is checked like any other, and its allocate clause feeds the unified allocation set"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - allocation
---

An `allocation def` — declared at package level or nested in a `part def` body — shall be
synthesized into a native `AllocationDef` element carrying `supertype:` (its `:>` clause) and `doc`.
An ingested `allocation` usage's `typedBy:` shall then be resolved and checked exactly like any other
`typedBy:` reference: an unresolved one raises `E111`.

A named `allocation` usage's `allocate <source> to <target>` clause shall be lifted onto the
synthesized `Allocation` as `allocatedFrom: [<source>]` / `allocatedTo: [<target>]` (GH #144), so
the ingested allocation contributes an edge to the §12.9 unified allocation set (`E314`, `W034`,
`matrix --allocations`). Endpoints are resolved against the full model at ingest time: the head
(a possibly `::`-qualified name) innermost-scope-first from the allocation's owning namespace out
to the model root; each further `.`-chain segment as a feature of the element reached so far,
declared on it or inherited through its `typedBy:`/`supertype:` chain. An unresolvable chain tail
is truncated to the deepest resolved prefix with `W542`; an unresolvable head is kept verbatim
(`.` → `::`) for `E502`/`E503` to report; a stable id is kept as-is.

## Rationale

`ElementType::AllocationDef` already existed in the native schema (SysMLv2 `allocation def`,
§2 element mapping), but ingestion mapped only `AllocationUsage`. An ingested allocation's
`typedBy:` therefore could never resolve to an in-model definition, and the structural-reference
check had to exempt every ingested `Allocation` from `E111` — which also hid genuinely dangling
references (GH #142). Mapping the definition closes the gap and removes the exemption.

## Scope

- `AllocationDef` is reachable from `PackageBodyElement` and `PartDefBodyElement` (not from
  `PartUsageBodyElement`, which has no allocation variant at all in this parser version) — both
  dispatchers gain the arm.
- `AllocationDef.body` is the thin `DefinitionBody` shared with `FlowDef`; the doc comment is lifted
  by the same `flow_body_doc` helper (`REQ-TRS-SYSMLV2-024`), including its
  `OccurrenceMember(OccurrenceBodyElement::Doc)` shape.
- The `allocate <source> to <target>` clause is lifted only for a **named** `allocation` usage;
  the anonymous `allocate a to b;` statement has no identity to synthesize an `Allocation` element
  against and stays unmapped. The optional `end ::>` end-name prefix is ignored (the parser
  discards it).
- Unlike `connect` lifting (`REQ-TRS-SYSMLV2-013`, a purely local AST lookahead), allocation
  endpoints resolve in a post-merge pass over the complete element list, because they routinely
  cross packages and name features inherited from a usage's type.
