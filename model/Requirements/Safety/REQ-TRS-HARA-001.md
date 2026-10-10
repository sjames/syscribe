---
type: Requirement
id: REQ-TRS-HARA-001
name: "W800 is not raised for a hazardous event that evaluates to QM"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
  - hara
---

`W800` ("HazardousEvent not referenced by any SafetyGoal") shall not fire for a `HazardousEvent` whose ASIL, derived from severity, exposure and controllability per ISO 26262-3 Table 4, is QM, because no safety goal is required for it.

## Behavior

- Applies only when severity, exposure and controllability are all present and parse; otherwise the ASIL is unknown and `W800` still fires.
- An event that evaluates to A–D and has no goal still raises `W800`.
- An event whose S, E or C is 0 evaluates to QM and is therefore also exempt.
