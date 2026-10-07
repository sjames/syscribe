---
type: Requirement
id: REQ-TRS-SYSMLV2-086
name: "Metadata applications on an element body are ingested into the native metadata: list"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A `@T { k = v; }`, `@T;`, `@n : T { ... }` or standalone `#T;`/`#T { ... }` member of a definition or usage body whose type is not one of the reserved `@Syscribe*` annotations shall be ingested as an entry of that element's native `metadata:` list (spec 3.8): `type:` is the written type reference, rewritten to the qualified name of the ingested `MetadataDef` it resolves to innermost-scope-first (kept as written otherwise, so the existing `E317`/`E318`/`W045` rules apply exactly as to a hand-authored element); an optional `name:` carries the declared name; each `k = v;` body member becomes a tagged value whose literal integer, real, boolean or string value keeps its type and whose other expression is carried as its rendered text. A package-level `#T` prefix member applies to the package member that follows it. The `@Syscribe*` annotations keep their existing field lift and never appear in `metadata:`.
