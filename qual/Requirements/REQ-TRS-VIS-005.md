---
id: REQ-TRS-VIS-005
type: Requirement
name: The IBD generator derives the boundary, owned part usages, ports with direction and connection edges from a PartDef or Part subject, and never invents a port
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: IBD` whose subject is a `PartDef` or `Part` (`ItemDef`/`Item` on
the same terms), the generator **shall** produce:

- a `Boundary` node for the subject carrying the subject's own port features as `Port`
  children; a `Part` subject **shall** also carry the ports its definition declares;
- one `Block` child of the boundary per owned part usage — an inline `features:` entry typed
  by a `PartDef`/`ItemDef`, or a child `Part`/`Item` element — labelled `name : Type` with a
  non-`1` multiplicity appended as `[n]`, with that usage's ports (its own port features, then
  the ports its definition declares) as `Port` children, each carrying the `direction` of the
  port feature it came from and no fixed side unless pinned;
- a `Connection`, `Flow`, `Binding` or `Succession` edge for each entry of the subject's
  `connections:`, `flowConnections:`, `bindingConnections:` or `successionConnections:`,
  resolving each endpoint's dotted feature chain against the diagram's own nodes exactly as
  the graph builder does (`engine.powerOut` is port `powerOut` of usage `engine`; a single
  segment is a usage or a boundary port), labelled with the entry's `name:` or the short name
  of its `typedBy:`, with deterministic edge ids and hierarchical layout hints.

A chain whose endpoint is not a node of the diagram **shall** produce no edge; the generator
**shall never** invent a port. `include:`/`exclude:` name owned part usages; an entry naming
none **shall** raise `W417`. A subject of any other type **shall** raise `W418` and produce no
nodes.

**Source:** `REQ-TRS-VIS-005` (product model).

**Acceptance criteria:** on the shared derive fixture, (a) a `PartDef` subject
(`Sys::PowerSystem`) yields a boundary with stereotype `part def`, a boundary port `mainOut`
with direction `out`, exactly the blocks `engine : Engine`, `motor : Motor [2]` and
`aux : Motor` parented to the boundary, port `engine.powerOut` (`out`, element ref
`Sys::PowerSystem::engine::powerOut`) and `aux.powerIn` (`in`) taken from the typing
definitions, and exactly a `Connection` edge `engine.powerOut → motor.powerIn` labelled
`PowerLink` with id `e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin`
followed by a `Binding` edge `motor.powerIn → mainOut`, with hierarchical layout hints; (b) a
`Part` subject (`Sys::PowerSystem::aux`) yields a boundary carrying its definition's `powerIn`
port with direction `in`; (c) `exclude: [motor]` removes the `motor` block and both edges that
touched `motor.powerIn`, with no finding; (d) `include: [engine, turbine]` leaves one block and
raises exactly one `W417` naming `turbine`; (e) a `Package` subject yields no nodes and a
`W418` naming the type; (f) a `connections:` entry to `ghost.powerIn` produces no edge and no
finding from the generator.
