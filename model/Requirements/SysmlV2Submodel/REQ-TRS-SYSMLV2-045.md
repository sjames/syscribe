---
type: Requirement
id: REQ-TRS-SYSMLV2-045
name: "A package-level SysMLv2 metadata def is ingested as a native MetadataDef"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - metadata
---

A `metadata def <Name> [:> <Super>] { ... }` member of a package shall become a `MetadataDef`
element (spec 8.15.1) carrying `supertype:` from the specialization, `isAbstract:` and the body's
`doc` text. A `metadata` *usage* has no sound native target and stays counted by `W543`; the
`@Syscribe*` annotations keep their existing dedicated lift.
