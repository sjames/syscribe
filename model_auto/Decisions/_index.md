---
type: Package
name: Decisions
---

Architecture Decision Records (ADRs) documenting the key design choices that shaped the
Engine ECU requirements tree and architecture. Each ADR justifies a decomposition or
allocation decision that would otherwise be implicit in the model structure.

The ADRs are listed by `syscribe show Decisions` (generated from this directory); `syscribe list ADR`
shows their status.

## ADR governance

Per Syscribe traceability rule R-002, every `Requirement` with `derivedFrom:` entries must
reference an `accepted` ADR in its `breakdownAdr:` field. ADRs in `proposed` status block
promotion of child requirements to `approved` (W303). New breakdowns require a new ADR
authored, reviewed, and accepted before the child requirements are created.
