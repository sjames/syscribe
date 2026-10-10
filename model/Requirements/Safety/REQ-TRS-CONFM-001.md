---
type: Requirement
id: REQ-TRS-CONFM-001
name: "ConfirmationMeasure.confirms accepts analysis and plan work products"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
---

`ConfirmationMeasure.confirms` shall accept the work products ISO 26262-2 and ISO/SAE 21434 confirmation reviews cover, not only goals, hazardous events and requirements (GH #233).

## Behavior

- Accepted targets: `SafetyGoal`, `CybersecurityGoal`, `HazardousEvent`, `Requirement`, and additionally `FaultTree`, `FMEASheet`, `TARASheet`, `ADR`, `Argument`, `TestPlan`, `Allocation`.
- `E851` (unresolved) is unchanged.
- `E860` still rejects any other element type and its message lists every accepted type.
- The `template ConfirmationMeasure` text lists the accepted types instead of "any model element ref".
