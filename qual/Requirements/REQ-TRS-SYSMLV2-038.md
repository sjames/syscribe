---
id: REQ-TRS-SYSMLV2-038
type: Requirement
name: Tool shall map supported element kinds to SysML v2 definitions and usages with supertype, typing, multiplicity, doc and satisfy
status: verified
reqDomain: software
verificationMethod: test
---

The export **shall** render Package, Part/PartDef, Port/PortDef, Attribute/AttributeDef, Connection/ConnectionDef, Interface/InterfaceDef, Item/ItemDef, RequirementDef and native Requirement (as `requirement def`), and header-plus-doc Action/State/Constraint/Calculation, with `:>`, `:`, `[n]`, `abstract`, `doc /* */` and `satisfy`.

**Source:** `REQ-TRS-SYSMLV2-038` (product model), `ADR-SYS-SYSMLV2-002`.
