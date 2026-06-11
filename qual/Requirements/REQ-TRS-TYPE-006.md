---
id: REQ-TRS-TYPE-006
type: Requirement
title: "Tool shall recognise and validate the AnalysisCase element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `AnalysisCase` usage element. (Its definition `AnalysisCaseDef` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the usage.)

- **SysMLv2 mapping:** `AnalysisCase` maps to `analysis` (§2.2) and is the usage of an `AnalysisCaseDef` (`analysis def`, §8.12.2). It shares the common case fields of §8.12.1: `subject:`, `actors:`, `objectives:`, `result:`, plus common action fields (`subActions:`, `successionConnections:`) and `typedBy:` identifying the AnalysisCaseDef it invokes.
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `typedBy:` for typing. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All case/action fields are optional.
- **Recognition behaviour:** a file with `type: AnalysisCase` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application for the definition follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]], [[REQ-TRS-TYPE-005]].

**Source:** §2.2, §8.12.1, §8.12.2

**Acceptance criteria:** A model containing a minimal `AnalysisCase` (with its `AnalysisCaseDef`) parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
