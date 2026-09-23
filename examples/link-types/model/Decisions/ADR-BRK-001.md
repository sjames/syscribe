---
type: ADR
id: ADR-BRK-001
name: "Brake-by-wire stopping requirement broken down into lock detection, pressure release and regeneration"
status: accepted
tags:
  - brakes
---

## Context

`REQ-BRK-001` states the vehicle-level stopping obligation. It spans a
software control loop, a hydraulic actuator and an energy-recovery strategy,
each owned by a different team and verified differently.

## Decision

Break `REQ-BRK-001` into three leaf requirements: wheel-lock detection
(`REQ-BRK-002`, software), hydraulic pressure release (`REQ-BRK-003`,
hardware) and regenerative braking share (`REQ-BRK-004`, software).

## Consequences

`REQ-BRK-004` competes with `REQ-BRK-002` for control of the wheel torque;
the two record that tension with the project's `conflictsWith` link type
rather than in prose alone, so `follow` and `impact` can find it.
