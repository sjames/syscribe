---
type: TestPlan
id: TP-GTI-EXEC-US-001
name: "Executable scenarios — US regression"
status: approved
scope: regression
configurations: [CONF-GTI-US-M3-001]
testCases: [TC-GTI-PROT-US-001, TC-GTI-MPPT-001]
---

Run with `syscribe simulate`, `mutate` and `inject` on every change to the protection chain or the MPPT machine.
