---
type: Requirement
id: REQ-TRS-VIS-004
name: "The BDD generator derives blocks, inheritance, composition and compartments from a Package or definition subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - bdd
---

For a derived `diagramKind: BDD` whose subject is a `Package`, `PartDef` or `ItemDef`, the
generator shall produce:

- one `Block` node per `PartDef`, `ItemDef`, `PortDef`, `InterfaceDef` or `ConnectionDef` that
  is a direct member of the subject package, or a direct sub-definition of the subject
  definition (the subject itself included), with the SysMLv2 stereotype (`part def`,
  `item def`, …) and the abstract flag;
- one `Compartment` child per block listing its `features:` (name, type, unit) and declared
  ports;
- an `Inheritance` edge from each block to the block its `supertype:` resolves to;
- a `Composition` edge from a block to each block that types one of its part usages
  (`features:` entries or child `Part` elements with a resolving `typedBy:`);
- an `Association` edge for each `ConnectionDef` whose two end types resolve to blocks on the
  diagram.

An edge is emitted only when both ends are nodes of the diagram (after `include:`/`exclude:`).
Layout hints shall be layered, top-to-bottom, with inheritance edges oriented so that a
supertype is placed above its subtypes.

## Rationale

The BDD is the SysML view every reader knows first, and inheritance plus composition are what
a hand-listed manifest most often gets wrong or leaves out.

## Scope

- Golden IR snapshot tests on a fixture package covering each bullet above.
- Element kinds outside the list (actions, states, requirements) do not appear on a BDD even
  if they are members of the subject.
