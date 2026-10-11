---
id: SG-GTI-004
type: SafetyGoal
status: approved
name: Do not start a fire through overheating
silLevel: 1
safeState: Output reduced, then bridge off
ftti: 10s
hazardousEvents:
  - HE-GTI-004
---

Heatsink or magnetics overheating in a closed enclosure on a roof or wall is a fire risk. The inverter **shall** derate and then shut down before components exceed limits.
