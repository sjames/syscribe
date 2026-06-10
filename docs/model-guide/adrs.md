# Architecture Decisions

`GUIDE · ADRS`

Architecture Decision Records (ADRs) document the rationale behind every significant modeling decision. In Syscribe they are first-class elements with a stable `ADR-*` identifier.

## ADR element schema

```yaml
---
type: ADR
id: ADR-SYS-SAFE-001
title: Decompose safety goal into FC fault detection and safe landing sub-requirements
status: accepted
---

## Context

The top-level safety requirement REQ-UAV-SAFE-000 is too broad to assign to a single
architecture element. It needs to be split into concrete, measurable sub-requirements.

## Decision

Decompose REQ-UAV-SAFE-000 into:
- REQ-UAV-FC-001 (flight controller fault detection ≤ 50 ms)
- REQ-UAV-SAFE-001 (autonomous safe landing on battery-critical event)

Both are assigned to software elements in the avionics bay.

## Consequences

The flight controller becomes the primary safety-relevant software element (SIL 3 / ASIL C).
A dedicated safe-landing action must be formally verified.
```

## Required fields

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `ADR(-[A-Z0-9]{2,12})+-[0-9]{3,8}` |
| `title` | Yes | Short description of the decision |
| `status` | Yes | `proposed` · `accepted` · `deprecated` · `superseded` |

## Lifecycle

```
proposed → accepted → deprecated
                   ↘ superseded (by a newer ADR)
```

An ADR is `proposed` while under review. It must be `accepted` before it can be cited in `breakdownAdr:`. Citing a `proposed` ADR from an `approved` requirement fires **W303**.

## Where ADRs are required

The validator (rule §12.2) requires a `breakdownAdr:` on every requirement that has `derivedFrom:`. This means every time you split a parent requirement into children, you must:

1. Create an ADR explaining why the split is correct
2. Set the ADR to `accepted`
3. Reference it from each child: `breakdownAdr: Decisions::MyADR`

## ID convention

ADR IDs encode the domain and a sequence number:

| Example ID | Scope |
|---|---|
| `ADR-SYS-001` | System-level decision |
| `ADR-SW-SCHED-001` | Software scheduler decision |
| `ADR-HW-PWR-001` | Hardware power subsystem decision |
| `ADR-SYS-SAFE-001` | System-level safety decision |

Keep segment lengths between 2 and 12 uppercase alphanumeric characters.
