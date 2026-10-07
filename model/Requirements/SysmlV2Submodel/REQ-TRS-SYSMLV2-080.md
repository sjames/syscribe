---
type: Requirement
id: REQ-TRS-SYSMLV2-080
name: "Views, viewpoints, renderings and other already-mapped kinds nested in a part usage become native elements"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A `view`, `view def`, `viewpoint`, `viewpoint def`, `rendering`, `rendering def`, `constraint def`, `calc def`, `calc`, `metadata def`, `use case` and `verification` member nested in a `part` usage body (reachable in 0.57, a parse failure in 0.54) shall be ingested into the same native element types as the package-level and part-def forms, qualified under the part usage.
