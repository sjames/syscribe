---
type: Package
name: SysmlV2Submodel
---

Requirements for native SysML v2/KerML submodel ingestion: letting a directory inside the model
tree hold real `.sysml`/`.kerml` textual files, parsed in-process and merged into Syscribe's
traceability graph as first-class elements, with cross-references running in both directions
between that submodel and native Syscribe elements.

All requirements derive from `REQ-TRS-SYSMLV2-000` and are governed by `ADR-SYS-SYSMLV2-001`
(`Decisions::SysmlV2SubmodelADR`): marking a package as a submodel via `sysmlSubmodel: true`
(`REQ-TRS-SYSMLV2-001`), native parsing and qname-mapped merge into the graph
(`REQ-TRS-SYSMLV2-002`), a SysMLv2 element's `satisfy`/`verify` targeting a native `Requirement`
(`REQ-TRS-SYSMLV2-003`), a native `TestCase`'s `verifies:` targeting a SysMLv2 element
(`REQ-TRS-SYSMLV2-004`), a SysMLv2 variation point targeting a native `FeatureDef`
(`REQ-TRS-SYSMLV2-005`), graceful degradation under a dedicated error/warning code range
(`REQ-TRS-SYSMLV2-006`), the parse-broad/map-narrow element-coverage boundary
(`REQ-TRS-SYSMLV2-007`), a fixed set of `@Syscribe*` metadata annotations for
`domain:`/integrity-level/`shortName:`/`implementedBy:` (`REQ-TRS-SYSMLV2-008`), `doc /* ... */`
comment lifting into the synthesized element's `doc` body (`REQ-TRS-SYSMLV2-009`), a named
connection usage's `connect` endpoints lifting onto the owning part's `connections:` field
(`REQ-TRS-SYSMLV2-010`), and `n2`'s scoped subpart axis including SysMLv2-synthesized children by
qname containment, not only `features:`-declared ones (`REQ-TRS-SYSMLV2-011`), and a named
connection usage's own trailing `doc /* ... */` body lifting onto the synthesized `Connection`
element (`REQ-TRS-SYSMLV2-012`), and resolving a two-segment `connect` endpoint to a redeclared
nested feature of its head when one is actually declared, falling back to the head-only edge
otherwise (`REQ-TRS-SYSMLV2-013`), and a doc-comment-embedded `@Syscribe*` directive syntax
lifting the same fixed field set onto `interface def`/`port def`/`connection def` — element kinds
whose body grammars carry no `MetadataAnnotation` slot for the real `@Name{...}` form
(`REQ-TRS-SYSMLV2-014`).

This is a read-only validator: the submodel's `.sysml`/`.kerml` files stay authoritative and are
edited by their own native tooling, never by Syscribe's web UI or mutate commands. A
writer/serializer back into SysML v2 text, two-way round-trip authoring, and full SysML v2 static
semantic validation (type-checking, multiplicity legality, standard-library-aware inheritance) are
explicitly out of scope, tracked as follow-on only if a concrete need arises.
