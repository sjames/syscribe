---
type: Requirement
id: REQ-TRS-SYSMLV2-029
name: "A SysMLv2 allocation def maps to the native AllocationDef, so an ingested allocation usage's typedBy: resolves and is checked like any other"
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
- An allocation usage's `allocate <source> to <target>` clause is **not** lifted onto
  `allocatedFrom:`/`allocatedTo:` by this requirement — a separate, larger follow-on (it needs
  feature-chain endpoint resolution, like `connect`).
