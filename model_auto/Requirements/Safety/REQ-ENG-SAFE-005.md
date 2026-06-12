---
type: Requirement
id: REQ-ENG-SAFE-005
name: Throttle close command shall be verified by position feedback within 200 ms
status: approved
reqDomain: software
asilLevel: A
verificationMethod: test
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-003
---

After the Engine ECU issues a throttle-close command (target position ≤ 5 %),
the safety monitor **shall** verify that the throttle position sensor (TPS)
reading falls below 15 % within 200 ms.

If the TPS reading remains at or above 15 % after the 200 ms verification
window, the safety monitor **shall**:

1. Assert a fuel-cut on all cylinders (immediate, within one engine cycle).
2. Set Diagnostic Trouble Code **P2111** (Throttle Actuator Control System Stuck
   Open) in the DTC memory.
3. Illuminate the malfunction indicator lamp (MIL) and set the engine to
   limp-home mode (idle torque only, throttle target = 7 %).

The 200 ms window is derived from the FTTI of safety goal SG-ENG-003. The 15 %
threshold allows for hysteresis and mechanical settle time of the return spring,
while still detecting a genuinely stuck-open plate (which would remain above
20 % under any air-demand condition).

The verification logic **shall** use the average of both TPS tracks (TPS-A and
TPS-B) as the measured position, provided both tracks agree within 5 %. If the
tracks diverge by more than 5 %, the diagnostic established by REQ-ENG-SAFE-001
takes precedence and the fuel-cut is asserted immediately, regardless of the
200 ms window.

Compliance evidence: hardware-in-the-loop test TC-ENG-SAFE-006 injecting a
stuck-open actuator fault and measuring time from close command to fuel-cut
assertion.
