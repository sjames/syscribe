---
id: REQ-TRS-SYSMLV2-085
type: Requirement
name: "W543 and the documentation name exactly the constructs that remain unmapped"
status: verified
reqDomain: software
verificationMethod: test
---

After `REQ-TRS-SYSMLV2-083`/`-084` the `W543` advisory shall count only constructs with no sound native target (`actor`, package-level `filter`, `metadata` usages, KerML declarations, anonymous `dependency`, `occurrence` with a portion kind, root-level `alias`, an unresolved package-level `satisfy`/`include`), and `docs/model-guide/sysmlv2-submodel.md` section 6 and the `W543` rows of the validation catalogues state that list and the reason for each, without claiming any limit that no longer holds.

**Source:** `REQ-TRS-SYSMLV2-085` (product model), `ADR-SYS-SYSMLV2-001`.
