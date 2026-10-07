---
id: REQ-TRS-SYSMLV2-089
type: Requirement
name: "A while loop keeps its until condition"
status: verified
reqDomain: software
verificationMethod: test
---

`while c { ... } until d;` shall be ingested as a `LoopAction` with `loopKind: while`, `condition: c` and the additive `untilCondition: d` sub-field (spec 8.7.6), and `export-sysml` shall write such an entry back as that statement.

**Source:** `REQ-TRS-SYSMLV2-089` (product model), `ADR-SYS-SYSMLV2-001`.
