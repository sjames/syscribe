---
id: REQ-TRS-SYSMLV2-071
type: Requirement
name: "A repository regression test bounds the number of behavioural entries export-sysml cannot write"
status: verified
reqDomain: software
verificationMethod: test
---

A repository-level test shall export every `.md`-native behavioural element (ActionDef/Action/StateDef/State) under `model/` and `examples/` and count the entries emitted as `not exported` comments. It shall fail when the count exceeds a recorded budget, and when the count falls below the budget (so the budget is ratcheted down whenever fidelity improves), so a regression in ingestion or export surfaces in CI.

**Source:** `REQ-TRS-SYSMLV2-071` (product model), `ADR-SYS-SYSMLV2-002`.
