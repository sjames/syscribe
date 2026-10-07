---
type: Requirement
id: REQ-TRS-SYSMLV2-098
name: "Bare root-level definitions and usages merge under the submodel's anchor package"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A definition or usage declared at the root of a `.sysml`/`.kerml` file, outside every `package` (`part def X;`, `requirement def R;`, `part p : X;`, ...), shall be merged under the `sysmlSubmodel:` anchor package exactly as if it were a member of a package with the anchor's qualified name: it becomes a first-class element with qualified name `<anchor>::X`, carrying every field the same member inside a package carries, across every file of the subtree, and `export-sysml` writes it back as a direct member of the anchor package's body. The other root-level member forms follow the same rule — a `#T` prefix applies to the member that follows it, a package-level `satisfy R by X;` lifts onto `X`, a `metadata`/`@T` application lifts onto the anchor package's own element (or its resolvable `about` target), and a `doc` comment appends to the anchor's documentation. A named root-level `alias` keeps `REQ-TRS-SYSMLV2-095`'s lift. `W543` no longer counts a `root-level member`; a root-level member counts under exactly the kind it would count under inside a package (`actor`, `filter`, `KerML declaration`, ...).
