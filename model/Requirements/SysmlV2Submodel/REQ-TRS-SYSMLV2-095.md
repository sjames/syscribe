---
type: Requirement
id: REQ-TRS-SYSMLV2-095
name: "A root-level alias lifts onto the submodel's anchor package"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A named `alias n for X;` at the root of a `.sysml`/`.kerml` file, outside every package, shall be lifted into the `aliases:` list of the `sysmlSubmodel:` anchor package's own element (`{name, shortName?, for}`, the same shape `REQ-TRS-SYSMLV2-043` gives a package-level alias) and no longer counts in `W543`. Every other bare root-level member merges under the anchor package as of `REQ-TRS-SYSMLV2-098` (before it, it stayed unmapped and was counted as `root-level member`).
