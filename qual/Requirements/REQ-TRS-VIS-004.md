---
id: REQ-TRS-VIS-004
type: Requirement
name: The BDD generator derives blocks, compartments, inheritance, composition and association edges from a Package or definition subject
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: BDD` whose subject is a `Package` (or `LibraryPackage`/
`Namespace`), `PartDef` or `ItemDef`, the generator **shall** produce:

- one `Block` node per `PartDef`, `ItemDef`, `PortDef`, `InterfaceDef` or `ConnectionDef`
  that is a direct member of the subject package — or, for a definition subject, the subject
  itself plus its direct sub-definitions — carrying the SysMLv2 stereotype (`part def`,
  `connection def`, …) and the abstract flag; members of any other kind (an `ActionDef`, a
  state, a requirement) **shall not** appear;
- one `Compartment` child per block that has any, listing attribute features as
  `name : Type [unit]` and port features as `port name : Type (direction)`;
- an `Inheritance` edge from each block to the block its `supertype:` resolves to;
- a `Composition` edge from a block to each block that types one of its part usages — an
  inline `features:` entry typed by a definition, or a child `Part`/`Item` element — labelled
  with the usage name and, when not `1`, its multiplicity (`motor [2]`);
- an `Association` edge for each `ConnectionDef` on the diagram whose first two
  `ends[].typedBy` resolve to blocks on the diagram, labelled with the connection definition's
  name.

An edge **shall** be emitted only when both of its ends are nodes of the diagram after
`include:`/`exclude:` are applied. Edge ids **shall** be deterministic. A subject of any other
type **shall** raise `W418` and produce no nodes.

**Source:** `REQ-TRS-VIS-004` (product model).

**Acceptance criteria:** on the shared derive fixture (`Sys` with an abstract `Base`, `Engine`
and `Motor` specialising it, a `PowerPort`, a `PowerLink` connection definition, a composed
`PowerSystem` with inline usages and a child `aux` part, and an `ActionDef`), (a) a package
subject yields exactly the six definition blocks in qualified-name order and never the
`ActionDef`, `Base` is abstract with stereotype `part def`, `PowerLink` has stereotype
`connection def`, `Engine`'s compartment lists `mass : Real [kg]` and
`port powerOut : PowerPort (out)`, and the edges are exactly two inheritance, three
composition (`engine`, `motor [2]`, `aux`) and one association (`PowerLink`), with the
composition edge id `e-composition-s-sys-powersystem-s-sys-motor-motor`; (b) a `PartDef`
subject with no sub-definitions yields one block and no edges; (c)
`include: [Engine, Sys::Base, Ghost]` yields `Base` and `Engine` with only their inheritance
edge and exactly one `W417` naming `Ghost`, and `exclude: [Sys::Base]` removes the block and
both inheritance edges with no finding; (d) an `ActionDef` subject yields no nodes and one
`W418` naming the type.
