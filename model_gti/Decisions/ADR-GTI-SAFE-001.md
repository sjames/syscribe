---
type: ADR
id: ADR-GTI-SAFE-001
name: "Give the hardware trip its own voltage sensing chain"
status: accepted
date: "2026-10-11"
tags: [safety, architecture, fault-injection]
---

## Context

The first version of the executable protection chain fed the software grid monitor and the hardware trip from one sensing chain. `syscribe inject` reported a single-fault defeat of the invariant "after a grid loss the grid is never energised for longer than 2 s": with the sensor reading nominal voltage, neither disconnection path saw the loss (FM-GTI-009, safety goal SG-GTI-001). The architecture claimed single-fault tolerance through two independent paths, but the sensing was a shared single point.

## Decision

The hardware trip reads the grid voltage through its own chain (REQ-GTI-HW-008). The composite has two sensing parts, `sensorSw` and `sensorHw`. A sensing fault on one chain no longer defeats the invariant (FM-GTI-009 and FM-GTI-011 both survive). A common cause that affects both chains (for example a wiring error at the grid connection point) stays a residual risk. It is modelled as FTE-GTI-009 in FT-GTI-001, a single cut set that the campaign reports as defeated (open decision).

## Consequences

One more isolated voltage measurement on the control board and its own reference. The start-up self-test must exercise the hardware trip (REQ-GTI-SYS-071).
