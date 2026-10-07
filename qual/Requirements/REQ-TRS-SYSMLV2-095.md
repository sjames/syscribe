---
id: REQ-TRS-SYSMLV2-095
type: Requirement
name: "A root-level alias lifts onto the submodel's anchor package"
status: verified
reqDomain: software
verificationMethod: test
---

A named `alias n for X;` at the root of a `.sysml`/`.kerml` file, outside every package, shall be lifted into the `aliases:` list of the `sysmlSubmodel:` anchor package's own element (`{name, shortName?, for}`, the same shape `REQ-TRS-SYSMLV2-043` gives a package-level alias) and no longer counts in `W543`. Every other bare root-level member stays unmapped and is counted as `root-level member`.

**Source:** `REQ-TRS-SYSMLV2-095` (product model), `ADR-SYS-SYSMLV2-001`.
