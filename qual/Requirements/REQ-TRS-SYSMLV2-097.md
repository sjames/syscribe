---
id: REQ-TRS-SYSMLV2-097
type: Requirement
name: "W543 and the documentation name exactly the constructs that remain unmapped after REQ-TRS-SYSMLV2-086..096"
status: verified
reqDomain: software
verificationMethod: test
---

After `REQ-TRS-SYSMLV2-086`..`-096` the `W543` advisory shall count only constructs with no sound native target — package-level `actor` (no actor element type; the AST keeps only the identification), package-level `filter` (no native target: `filter:` exists only on `View`/`expose`), KerML declarations (every KerML-only member kind), a `metadata` application whose `about` target does not resolve, an anonymous `alias` or one in an anonymous package, a `textual representation`, an unresolved package-level `satisfy`/`include`, and — counted as `other package member` — a package-level usage or relationship member with no package-level native target (`ref`, `connect`, `binding`, `succession`, `exhibit`, `include`, `expose`, `perform`, `assert constraint`, a keyword-less `name = expr;` binding), a user-defined-keyword declaration and grammar the parser marks unsupported — and `docs/model-guide/sysmlv2-submodel.md` section 6 and the `W543` rows of both validation catalogues shall state that list with the reason for each, plus the constructs the parser itself does not represent (a `#T` prefix inside a definition body), without claiming any limit that no longer holds. (A bare root-level member, counted as `root-level member` here, is mapped by `REQ-TRS-SYSMLV2-098`.)

**Source:** `REQ-TRS-SYSMLV2-097` (product model), `ADR-SYS-SYSMLV2-001`.
