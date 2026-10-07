---
id: REQ-TRS-SYSMLV2-021
type: Requirement
name: "A SysMLv2 viewpoint def maps to the native ViewpointDef schema — stakeholders, concerns; a viewpoint usage maps onto View"
status: verified
reqDomain: software
verificationMethod: test
---

A `viewpoint def` shall be synthesized into a native `ViewpointDef` element carrying the same
`stakeholders:`/`concerns:` shape a hand-authored one uses
(`model/Viewpoints/SystemsEngineerViewpoint.md`'s existing convention). A `viewpoint` usage shall be
synthesized into a native `View` element — the native schema has no dedicated `Viewpoint` usage
`ElementType`, and `View` is already documented as "usage of a ViewDef or ViewpointDef."

**Source:** `REQ-TRS-SYSMLV2-021` (product model), `ADR-SYS-SYSMLV2-001` addendum.
