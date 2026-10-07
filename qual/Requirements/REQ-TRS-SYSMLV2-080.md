---
id: REQ-TRS-SYSMLV2-080
type: Requirement
name: "Views, viewpoints, renderings and other already-mapped kinds nested in a part usage become native elements"
status: verified
reqDomain: software
verificationMethod: test
---

A `view`, `view def`, `viewpoint`, `viewpoint def`, `rendering`, `rendering def`, `constraint def`, `calc def`, `calc`, `metadata def`, `use case` and `verification` member nested in a `part` usage body (reachable in 0.57, a parse failure in 0.54) shall be ingested into the same native element types as the package-level and part-def forms, qualified under the part usage.

**Source:** `REQ-TRS-SYSMLV2-080` (product model), `ADR-SYS-SYSMLV2-001`.
