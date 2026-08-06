---
type: Requirement
id: REQ-RTH-001
name: "Automatic Return-to-Home on critical battery level"
status: approved
reqDomain: system
reqClass: system
tags:
  - rth
  - battery
---

The UAV shall automatically initiate a return-to-home flight path when
remaining battery capacity drops below the critical threshold defined for the
active flight profile.

## Rationale

This is the top-level stakeholder goal the `PlanningItem` breakdown in
`Planning::PI-RTH-001` exists to achieve (`achieves:`, `REQ-TRS-PLANITEM-003`).
It is broken into `REQ-RTH-002`/`REQ-RTH-003` via `ADR-RTH-001`.
