---
type: Package
name: Decisions
---

Architecture Decision Records (ADRs) documenting the key design choices that shaped the
Engine ECU requirements tree and architecture. Each ADR justifies a decomposition or
allocation decision that would otherwise be implicit in the model structure.

## Contents

| ID | Title | Status | Governs |
|---|---|---|---|
| `ADR-ENG-SYS-001` | Decompose system requirements into performance, safety, and security | accepted | `REQ-ENG-SYS-000` breakdown |
| `ADR-ENG-SAFE-001` | ASIL D decomposition into SW + HW sub-requirements | accepted | `REQ-ENG-SAFE-000` breakdown |
| `ADR-ENG-PERF-001` | Decompose performance into throttle response and fuel efficiency | accepted | `REQ-ENG-PERF-000` breakdown |

## ADR governance

Per Syscribe traceability rule R-002, every `Requirement` with `derivedFrom:` entries must
reference an `accepted` ADR in its `breakdownAdr:` field. ADRs in `proposed` status block
promotion of child requirements to `approved` (W303). New breakdowns require a new ADR
authored, reviewed, and accepted before the child requirements are created.
