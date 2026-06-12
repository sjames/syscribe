---
id: REQ-TRS-ELEM-001
type: Requirement
name: Tool shall recognise all element types defined in §2
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and correctly process all element types defined in the Syscribe format specification §2, including: `PartDef`, `Part`, `ItemDef`, `Item`, `OccurrenceDef`, `Occurrence`, `IndividualDef`, `Individual`, `PortDef`, `Port`, `InterfaceDef`, `Interface`, `ConnectionDef`, `Connection`, `AttributeDef`, `Attribute`, `EnumerationDef`, `ActionDef`, `Action`, `FlowDef`, `StateDef`, `RequirementDef`, `Requirement`, `TestCase`, `ADR`, `Allocation`, `ViewDef`, `View`, `Package`, `MetadataDef`, `UseCaseDef`, `AnalysisCaseDef`, `VerificationCaseDef`.

**Source:** §2.1–§2.6

**Acceptance criteria:** A model containing one element of each type passes without `E005` findings.
