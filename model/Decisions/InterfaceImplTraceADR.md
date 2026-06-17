---
type: ADR
id: ADR-SYS-IFACE-001
name: "Extend implementedBy to Interface and InterfaceDef elements"
status: accepted
tags:
  - traceability
  - interface
---

## Context

The `implementedBy:` field and its associated W023 validation currently apply only to
`Part` and `PartDef` elements. Interface definitions (header files, IDL contracts,
protocol specs) are equally important artifacts in the V-model trace chain —
`Requirement → satisfies → Architecture → implementedBy → Code` — yet there is no
first-class way to link an `Interface` or `InterfaceDef` to the source file that
implements it.

## Decision

Extend `implementedBy:` and the W023 path-existence check to cover `Interface` and
`InterfaceDef` elements. No new field is introduced; the existing semantics
(string or list, draft-suppressed, remote URIs exempt) apply unchanged.

## Consequences

- Engineers can link `InterfaceDef` elements directly to their header/IDL/source files.
- W023 fires for missing local paths on non-draft `Interface`/`InterfaceDef` elements,
  the same as for `Part`/`PartDef`.
- No breaking change: elements without `implementedBy:` are unaffected.
