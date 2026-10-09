---
type: Argument
id: ARG-ENG-001
name: Argue over independent safety monitoring and watchdog supervision
status: approved
argumentType: strategy
supports: SG-ENG-001
evidence:
  - REQ-ENG-SAFE-001
  - REQ-ENG-SAFE-002
---

Unintended acceleration is prevented by two supervisions that do not share a failure path: the
software safety monitor (detect and command the fail-safe state within 100 ms) and the hardware
watchdog (reset the ECU within 30 ms of a software failure).
