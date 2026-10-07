---
type: Requirement
id: REQ-TRS-SYSMLV2-087
name: "A metadata application with an about clause attaches to each resolvable target"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A package-level or body-level `@T about Y;`, `metadata m : T about Y;` or `metadata T about Y;` (a usage with no `:` typing names its type, as the grammar's required typing implies) shall be attached, one `metadata:` entry per target, to each `Y` that resolves innermost-scope-first against the ingested elements of the submodel. A target that does not resolve keeps the entry on the declaring holder with `about:` set to the written target and is counted once as `metadata` in `W543`. A package-level application with no `about` applies to the package itself.
