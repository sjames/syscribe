---
type: TestPlan
id: TP-GTI-EXEC-IN-001
name: "Executable scenarios — IN regression"
status: approved
scope: regression
appliesWhen: Features::Region::IN and Features::PvInput::Mppt3
configurations: [CONF-GTI-IN-M3-001]
testCases: [TC-GTI-PROT-IN-001, TC-GTI-MPPT-001]
---

Run with `syscribe simulate`, `mutate` and `inject` on every change to the protection chain or the MPPT machine.
