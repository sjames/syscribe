---
type: Requirement
id: REQ-TRS-SYSMLV2-093
name: "An anonymous dependency is ingested under a synthesized name"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

An anonymous `dependency from a to b;` shall be ingested as a `Dependency` element named `dependency_N` — N counting from 1 in source order within the owning scope, skipping a name already taken — with `clients:`/`suppliers:` resolved exactly as for a named one, so it no longer counts in `W543`; `export-sysml` writes the named form.
