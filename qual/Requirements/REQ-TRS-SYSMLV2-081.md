---
id: REQ-TRS-SYSMLV2-081
type: Requirement
name: "A succession's own name and multiplicities are ingested and exported"
status: verified
reqDomain: software
verificationMethod: test
---

`succession s first a then b;`, `succession s first a if g then b;` and the multiplicity forms `succession [m] first [x] a then [y] b;` shall be ingested into the `successionConnections:` entry as `name:` (an existing native sub-field) and the additive native `multiplicity:`/`afterMultiplicity:`/`beforeMultiplicity:` sub-fields (spec 8.4.4), and `export-sysml` shall write an entry carrying them as that statement. A succession type (`succession s : T first ...`) and the part-level `succession` between structural usages have no native target and are not mapped.

**Source:** `REQ-TRS-SYSMLV2-081` (product model), `ADR-SYS-SYSMLV2-001`.
