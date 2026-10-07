---
type: Requirement
id: REQ-TRS-SYSMLV2-071
name: "A repository regression test bounds the number of behavioural entries export-sysml cannot write"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A repository-level test shall export every `.md`-native behavioural element (ActionDef/Action/StateDef/State) under `model/` and `examples/` and count the entries emitted as `not exported` comments. It shall fail when the count exceeds a recorded budget, and when the count falls below the budget (so the budget is ratcheted down whenever fidelity improves), so a regression in ingestion or export surfaces in CI.
