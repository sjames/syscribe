---
type: Requirement
id: REQ-TRS-SYSMLV2-038
name: "The SysML v2 export maps packages, definitions and usages of the supported element kinds, with supertype, typing, multiplicity, doc and satisfy"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - mapping
---

The export shall render: `Package` as `package`, with the directory/qname tree as nested packages
(missing intermediate segments become implicit packages) and an element whose qname parent is a
definition or usage nested in that parent's body; `PartDef`/`Part`, `PortDef`/`Port`,
`AttributeDef`/`Attribute`, `ConnectionDef`/`Connection`, `InterfaceDef`/`Interface`,
`ItemDef`/`Item` as the matching `<kind> def`/`<kind>`; `RequirementDef` and native `Requirement` as
`requirement def`; and `ActionDef`/`Action`, `StateDef`/`State`, `ConstraintDef`/`Constraint`,
`CalculationDef`/`Calculation` as header-plus-doc declarations. A definition's `supertype:` renders as
`:>`, a usage's `typedBy:` as `:`, `multiplicity:` as `[n]`, `isAbstract: true` as `abstract`, and an
element's `satisfies:` as `satisfy <target>;` in its body. The element body text renders as
`doc /* ... */`; a `*/` inside the text shall be neutralized so it cannot close the comment early.
