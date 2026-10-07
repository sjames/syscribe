---
type: Requirement
id: REQ-TRS-SYSMLV2-036
name: "A package-level SysMLv2 doc comment lifts onto the synthesized Package element's doc text"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - doc
---

A `doc /* ... */` declared directly in a `package` body shall become the doc text of the
synthesized `Package` element for that package (several blocks, or blocks from several files that
merge into one package, are joined by a blank line; each block's own delimiter padding is trimmed
as for every other doc lift, `REQ-TRS-SYSMLV2-009`). A nested package gets only its own docs.
Package-level `doc` no longer counts toward `W543`.

`doc` on a `requirement def`/`requirement` body remains a separate follow-up.
