---
type: Requirement
id: REQ-TRS-SYSMLV2-085
name: "W543 and the documentation name exactly the constructs that remain unmapped"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

After `REQ-TRS-SYSMLV2-083`/`-084` the `W543` advisory counts only constructs with no sound native target (`actor`, package-level `filter`, `metadata` usages, KerML declarations, anonymous `dependency`, `occurrence` with a portion kind, root-level `alias`, an unresolved package-level `satisfy`/`include`), and `docs/model-guide/sysmlv2-submodel.md` section 6 and the `W543` rows of the validation catalogues state that list and the reason for each, without claiming any limit that no longer holds.
