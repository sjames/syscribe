---
id: SG-GTI-003
type: SafetyGoal
status: approved
name: Do not expose persons to array leakage current
silLevel: 2
safeState: Grid relays open, array disconnected
ftti: 300ms
hazardousEvents:
  - HE-GTI-003
---

Transformerless topologies couple the array to the grid; a ground fault or damaged insulation can put dangerous current through a person touching a module frame. The inverter **shall** check insulation before connecting and disconnect on residual current.
