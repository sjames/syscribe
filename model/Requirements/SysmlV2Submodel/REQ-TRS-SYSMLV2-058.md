---
type: Requirement
id: REQ-TRS-SYSMLV2-058
name: "Behaviour that ingestion would not read back identically is exported as a comment, never as text that re-ingests differently"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

When a native `subActions:`/`subStates:`/`transitions:` entry cannot be expressed so that the same ingestion produces an equal entry (an unknown `kind:`, a hand-chosen name on a construct whose name ingestion synthesizes, an expression that is not valid SysML v2 text, a transition with no resolvable shape), the exporter shall emit that entry as `// …` comment lines naming what was not exported, and shall never emit a best-effort approximation. The export report and ingestion stay unchanged.
