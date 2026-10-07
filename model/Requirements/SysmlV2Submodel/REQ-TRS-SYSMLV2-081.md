---
type: Requirement
id: REQ-TRS-SYSMLV2-081
name: "A succession's own name and multiplicities are ingested and exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`succession s first a then b;`, `succession s first a if g then b;` and the multiplicity forms `succession [m] first [x] a then [y] b;` shall be ingested into the `successionConnections:` entry as `name:` (an existing native sub-field) and the additive native `multiplicity:`/`afterMultiplicity:`/`beforeMultiplicity:` sub-fields (spec 8.4.4), and `export-sysml` shall write an entry carrying them as that statement. A succession type (`succession s : T first ...`) and the part-level `succession` between structural usages have no native target and are not mapped.
