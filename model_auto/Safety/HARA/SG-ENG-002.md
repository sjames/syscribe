---
type: SafetyGoal
id: SG-ENG-002
title: Prevent critical engine stall during high-speed operation
status: approved
asilLevel: B
safeState: Controlled deceleration with driver warning before stall
ftti: 500ms
hazardousEvents:
  - HE-ENG-002
---

The Engine ECU **shall** prevent uncontrolled engine stall during high-speed
operation by detecting crankshaft position sensor loss and initiating a
controlled deceleration sequence before the engine speed drops to the stall
threshold.

## Safe state

The safe state is controlled deceleration — the engine continues to provide
limited torque at idle via fuel cut and throttle-to-idle command, ensuring
the driver retains steering control and brake assistance.

## ASIL target

ASIL B — derived from HE-ENG-002 (S2 × E3 × C2).
