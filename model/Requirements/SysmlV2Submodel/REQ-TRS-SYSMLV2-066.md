---
type: Requirement
id: REQ-TRS-SYSMLV2-066
name: "Ingested multiplicity with a reversed or non-natural bound raises advisory W544"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Ingestion shall raise warning \`W544\` on a \`part\`/\`attribute\`/\`port\`/… usage whose multiplicity has integer bounds with lower greater than upper, a negative bound, or a non-integer numeric literal bound. A bound that is a name or other non-literal expression is not evaluated and raises nothing. The multiplicity text is stored unchanged.
