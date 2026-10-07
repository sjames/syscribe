---
type: Requirement
id: REQ-TRS-SYSMLV2-075
name: "Use case include accepts qualified targets and the declaring form"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

In a use case body, `include <qualified::Name>;` and the declaring form `include use case v : <qualified::Name>;` shall add the resolved qualified name of the referenced use case to `includes:` exactly as the simple-name reference form does (innermost scope first, an unresolved target counted in `W543` as `include`).
