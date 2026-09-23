---
type: Requirement
id: REQ-BRK-002
name: "Wheel-lock onset detected within 20 ms"
status: approved
reqDomain: software
reqClass: derived
derivedFrom: [REQ-BRK-001]
breakdownAdr: Decisions::ADR-BRK-001
links:
  conflictsWith: REQ-BRK-004
tags:
  - brakes
  - safety
---

The brake controller software shall detect the onset of wheel lock on any
wheel within 20 ms of the wheel-slip ratio exceeding the configured threshold.

## Notes

`conflictsWith: REQ-BRK-004` is the mirror of the link on `REQ-BRK-004`. The
pair forms a two-element cycle, which is legitimate because `conflictsWith`
is declared `acyclic = false`.
