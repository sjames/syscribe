---
id: SG-GTI-001
type: SafetyGoal
status: approved
name: Do not energise a de-energised grid
silLevel: 2
safeState: Pwm off and both grid relays open
ftti: 2s
hazardousEvents:
  - HE-GTI-001
---

A utility lineworker may touch a conductor believed de-energised. The inverter **shall not** keep feeding an islanded or isolated grid: after loss of grid it must stop switching and disconnect within the island-clearing time (2 s), tolerating any single fault.
