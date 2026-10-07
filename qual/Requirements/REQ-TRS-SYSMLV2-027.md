---
id: REQ-TRS-SYSMLV2-027
type: Requirement
name: "A SysMLv2 analysis def/analysis maps to the native AnalysisCaseDef/AnalysisCase schema — subject, actors, objectives, result"
status: verified
reqDomain: software
verificationMethod: test
---

An `analysis def` shall be synthesized into a native `AnalysisCaseDef` element carrying
`supertype:`/`subject:`/`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named `analysis`
usage shall be synthesized into a native `AnalysisCase` element carrying the same fields with
`typedBy:` in place of `supertype:`.

**Source:** `REQ-TRS-SYSMLV2-027` (product model), `ADR-SYS-SYSMLV2-001` addendum.
