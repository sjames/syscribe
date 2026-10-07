---
type: Requirement
id: REQ-TRS-SYSMLV2-097
name: "W543 and the documentation name exactly the constructs that remain unmapped after REQ-TRS-SYSMLV2-086..096"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

After `REQ-TRS-SYSMLV2-086`..`-096` the `W543` advisory shall count only constructs with no sound native target — package-level `actor` (no actor element type; the AST keeps only the identification), package-level `filter` (no native target: `filter:` exists only on `View`/`expose`), KerML declarations, a `metadata` application whose `about` target does not resolve, a bare root-level member other than `alias`, an anonymous `alias` or one in an anonymous package, a `textual representation`, and an unresolved package-level `satisfy`/`include` — and `docs/model-guide/sysmlv2-submodel.md` section 6 and the `W543` rows of both validation catalogues shall state that list with the reason for each, plus the constructs the parser itself does not represent (a `#T` prefix inside a definition body), without claiming any limit that no longer holds.
