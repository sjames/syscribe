---
type: Requirement
id: REQ-TRS-SYSMLV2-073
name: "SysMLv2 ingestion and export are behaviour-identical on the current sysml-v2-parser"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Syscribe shall build against the newest published `sysml-v2-parser` (0.57.0 or later) and shall produce, for every `.sysml` input and every exported model, exactly the elements, qualified names, fields, `W54x` findings and export text that it produced on the previously pinned 0.54.0, except where a requirement documents an improvement. The version reported by `syscribe sysml` and MCP `sysml_submodels` shall equal the `Cargo.toml` pin.
