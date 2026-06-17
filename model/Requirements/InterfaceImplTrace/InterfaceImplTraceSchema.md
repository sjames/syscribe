---
type: Requirement
id: REQ-TRS-IFACE-001
name: "Interface and InterfaceDef shall accept an optional implementedBy field"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-IFACE-000]
breakdownAdr: Decisions::InterfaceImplTraceADR
tags:
  - schema
  - traceability
---

An `Interface` or `InterfaceDef` element shall accept an optional `implementedBy:`
field. The field shall accept a single string or a list of strings, each identifying
a source artifact (local path or remote URI) that implements or defines the interface
contract.

The `implementedBy:` field is already supported on `Part` and `PartDef`; this
requirement extends the same field and semantics to interface-typed elements without
introducing a new field name or syntax.

Elements without `implementedBy:` shall remain fully valid.
