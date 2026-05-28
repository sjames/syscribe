---
type: SafetyGoal
id: SG-ENG-001
title: Prevent unintended engine acceleration
status: approved
asilLevel: D
safeState: Throttle at fail-safe position (7 % opening), engine at idle torque
ftti: 100ms
hazardousEvents:
  - HE-ENG-001
---

The Engine ECU **shall** prevent unintended engine acceleration by detecting any
single safety fault in the throttle control chain within 100 ms and transitioning
to the fail-safe state (throttle at 7 % opening).

## Safe state

The fail-safe state is defined as the throttle actuator driven to the return
spring position (≈7 % opening). At this position, the engine produces
insufficient torque for significant acceleration on level ground.

## ASIL target

ASIL D — derived from HE-ENG-001 (S3 × E4 × C2). Both the software safety
monitor (REQ-ENG-SAFE-001) and the hardware watchdog (REQ-ENG-SAFE-002) must
contribute independently to achieve this target.
