---
type: Requirement
id: REQ-RTH-003
name: "Return-to-Home shall not trigger on transient sensor noise"
status: approved
reqDomain: software
reqClass: derived
derivedFrom: [REQ-RTH-001]
breakdownAdr: Decisions::ADR-RTH-001
tags:
  - rth
  - robustness
---

The flight controller shall debounce battery-level readings over a rolling
window before initiating a return-to-home event, so that a single transient
sensor glitch cannot spuriously trigger the behavior.

## Rationale

Derived from `REQ-RTH-001` per `ADR-RTH-001`. Achieved by
`Planning::PI-RTH-BUGFIX-001` (`achieves:`, **qualified-name form** — the
sibling `REQ-RTH-002` above uses the id form, so both accepted target styles
are exercised across this example, per `REQ-TRS-PLANITEM-003`). Like
`REQ-RTH-002`, this stays a leaf with no `satisfies:`, so it also raises
`W300` — expected, see the README.
