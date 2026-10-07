---
type: Requirement
id: REQ-TRS-SYSMLV2-048
name: "Multiplicity, subsets and redefines on SysMLv2 part, attribute, port and item usages are ingested into the native fields"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - usage
---

A `part`, `attribute`, `port` or `item` usage that declares a multiplicity (`[2]`, `[0..*]`, `[*]`),
`:>`/`subsets` or `:>>`/`redefines` shall carry them in the native `multiplicity:` (normalized text:
`2`, `0..*`, `*`), `subsets:` (list) and `redefines:` fields. Targets go through the existing
`E112`/`E113` structural-reference checks and the scoped resolver, like `typedBy:`.
