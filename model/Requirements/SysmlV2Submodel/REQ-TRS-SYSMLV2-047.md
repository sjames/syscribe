---
type: Requirement
id: REQ-TRS-SYSMLV2-047
name: "doc on a SysMLv2 requirement def or requirement lifts onto the element's doc text"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - doc
---

`doc /* ... */` blocks in the body of a `requirement def` or `requirement` usage shall become the
element's doc text, exactly like every other doc lift (`REQ-TRS-SYSMLV2-009`): trimmed, joined by a
blank line, the `@Syscribe*:` directive handling unchanged.
