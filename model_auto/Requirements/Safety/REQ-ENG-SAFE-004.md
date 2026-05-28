---
type: Requirement
id: REQ-ENG-SAFE-004
title: Rev limiter shall cut fuel and retard ignition above 6500 rpm within 20 ms
status: approved
reqDomain: software
asilLevel: A
verificationMethod: test
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-003
---

The Engine ECU **shall** enforce a dual-mechanism rev limiter that operates
independently of throttle position feedback:

1. **Soft limiter (6200 rpm)** — The safety monitor shall apply ignition retard
   of up to −30° BTDC on a per-cylinder basis when the crankshaft speed exceeds
   6200 rpm. Retard shall be applied within one engine cycle (maximum 20 ms at
   3000 rpm, proportionally less at higher speed). This reduces torque output and
   provides a graduated response before the hard fuel-cut threshold.

2. **Hard limiter (6500 rpm)** — The safety monitor shall suppress the fuel
   injection pulse for each cylinder whose crank-position-based injection window
   opens while engine speed is above 6500 rpm. Suppression shall be asserted within
   20 ms (one crank event evaluation cycle) of the measured speed exceeding the
   threshold. Fuel injection shall be re-enabled cylinder-by-cylinder as speed
   drops below 6400 rpm (100 rpm hysteresis).

Both mechanisms **shall** derive their enable condition directly from the
crankshaft position sensor event counter and **shall not** use throttle position,
manifold pressure, or pedal demand as inputs to the enable decision. This ensures
the rev limiter remains effective during a stuck-open throttle fault.

The implementation shall be allocated to the `System::Software::SafetyMonitor`
software component. Freedom from interference from the `System::Software::ThrottleControl`
component shall be demonstrated by static analysis of shared data and execution
partitioning (AUTOSAR OS application separation).

Compliance evidence: hardware-in-the-loop test TC-ENG-SAFE-005 exercising the
rev limiter at 6200 rpm and 6500 rpm with throttle position sensor disconnected.
