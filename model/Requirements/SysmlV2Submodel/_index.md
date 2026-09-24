---
type: Package
name: SysmlV2Submodel
---

Requirements for native SysML v2/KerML submodel ingestion: letting a directory inside the model
tree hold real `.sysml`/`.kerml` textual files, parsed in-process and merged into Syscribe's
traceability graph as first-class elements, with cross-references running in both directions
between that submodel and native Syscribe elements.

All requirements derive from `REQ-TRS-SYSMLV2-000` and are governed by `ADR-SYS-SYSMLV2-001`
(`Decisions::SysmlV2SubmodelADR`). The scope covers marking a package as a submodel
(`sysmlSubmodel: true`), native parsing and qname-mapped merge into the graph, cross-references in
both directions (SysMLv2 `satisfy`/`verify` to native requirements, native `verifies:` to SysMLv2
elements, SysMLv2 variation points to native `FeatureDef`s), graceful degradation under a dedicated
code range, the parse-broad/map-narrow coverage boundary, the `@Syscribe*` metadata annotations
(and their doc-comment form for kinds without a metadata slot), `doc` comment lifting, connection
and flow lifting onto the owning part, scoped `typedBy:` resolution, and the per-construct mappings
onto native schemas — state machines, actions, views/viewpoints/renderings, concerns, flows, enums
and the case family (case, analysis, verification).

The member requirements are listed by `syscribe show Requirements::SysmlV2Submodel` (generated
from this directory — not maintained here).

This is a read-only validator: the submodel's `.sysml`/`.kerml` files stay authoritative and are
edited by their own native tooling, never by Syscribe's web UI or mutate commands. A
writer/serializer back into SysML v2 text, two-way round-trip authoring, and full SysML v2 static
semantic validation (type-checking, multiplicity legality, standard-library-aware inheritance) are
explicitly out of scope, tracked as follow-on only if a concrete need arises.
