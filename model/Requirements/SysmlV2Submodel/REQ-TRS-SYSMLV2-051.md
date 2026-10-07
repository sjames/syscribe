---
type: Requirement
id: REQ-TRS-SYSMLV2-051
name: "export-sysml turns satisfies on non-part elements into real satisfy statements and verifies on requirements into verify statements"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - traceability
---

An exported element that is not a `Part`/`PartDef` and has `satisfies:` shall export one real
package-level `satisfy <requirement> by <element>;` statement in its enclosing package instead of
the `// satisfies:` comment; a `RequirementDef`/`Requirement` with `verifies:` shall export
`verify <target>;` members in its body, the only context the parser (and so ingestion) accepts.
Anywhere SysML has no `verify` form, the link stays a comment. Re-ingesting the output yields the
same `satisfies:`/`verifies:` links.
