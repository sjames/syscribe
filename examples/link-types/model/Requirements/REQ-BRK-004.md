---
type: Requirement
id: REQ-BRK-004
name: "Regenerative braking share maximised during service braking"
status: approved
reqDomain: software
reqClass: derived
derivedFrom: [REQ-BRK-001]
breakdownAdr: Decisions::ADR-BRK-001
links:
  conflictsWith: REQ-BRK-002
tags:
  - brakes
  - energy
---

The brake blending software shall supply at least 70 percent of the requested
deceleration torque through regenerative braking during service braking below
0.3 g, whenever the traction battery can accept the recovered energy.

## Notes

Maximising regenerative torque on the driven axle raises its slip ratio, which
is exactly what `REQ-BRK-002` watches for — hence `conflictsWith`.
