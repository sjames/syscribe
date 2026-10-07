---
type: Requirement
id: REQ-TRS-SYSMLV2-044
name: "A SysMLv2 library package or namespace is ingested as a Package, at the root and nested"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - package
---

`library package` and `namespace` declarations shall be merged and converted exactly like a
`package`: a named one becomes a `Package` element with its members nested under it (same-named
declarations across files merge), at the root of a file and nested inside another package. The
`standard` marker of a `library package` is not preserved. Neither kind counts toward `W543` any
more.
