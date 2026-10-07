---
id: REQ-TRS-VIS-022
type: Requirement
name: The Allocation generator derives logical-to-physical allocation maps with swimlanes from a Package, AllocationDef or Allocation subject
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: Allocation` whose subject is a `Package` (or `LibraryPackage`/
`Namespace`), `AllocationDef` or `Allocation`, `vis::derive::allocation` **shall** collect every
allocation pair under the subject — an `Allocation` element's top-level
`allocatedFrom:`/`allocatedTo:`, each `features:` entry of `type: Allocation` with those two
fields, each `allocations:` entry of an `AllocationDef`, and each `allocatedTo:` on a
`Part`/`PartDef`/`Action`/`ActionDef` under the subject (its source being the element itself) —
and **shall** produce:

- two `Swimlane` nodes, `logical` and `physical`, containing one `Block` node per distinct
  source and per distinct target element respectively, each drawn with its real type's
  stereotype (an unresolved end becomes a dashed block labelled by the reference text); an
  element that is both a source and a target is drawn once in each lane;
- one `Allocation` edge per pair from the source block to the target block, labelled by the
  allocation usage's `name` when present, with the `«allocate»` keyword from `vis::style`.

`include:`/`exclude:` **shall** apply to the end elements (by qualified name, stable id or
short name), an entry naming nothing being `W417`. Shape ids **shall** be
`derived_shape_id(<element qualified name>)` and the lanes `<subjectId>-logical` and
`<subjectId>-physical`. A subject of any other type **shall** be `W418` with an empty graph.
Layout hints **shall** be layered, left-to-right, hierarchical. The Mermaid and SVG writers
**shall** accept the generated graph (PlantUML has no mapping for the kind).

**Source:** `REQ-TRS-VIS-022` (product model).

**Acceptance criteria:** through the real walker and validator on a fixture `Allocations`
package with an `Allocation` element using `features:` pairs (one target unresolved), an
`AllocationDef` with `allocations:` and a `PartDef` with its own `allocatedTo:`: (a) a package
subject yields the two lanes named after the subject's id, every source in the logical lane and
every target in the physical lane with real-type stereotypes, the unresolved end as a dashed
block labelled by its reference text, the element on both sides once per lane, and one
`«allocate»` edge per pair labelled by its usage name, matching the golden IR
`tests/vis_snapshots/derived/alloc.json`; (b) `include:` by short and qualified name keeps only
the named ends and the edges joining them with one `W417` for a stray entry, and `exclude:`
drops an end and its edges; (c) a `PartDef` subject is `W418` with an empty graph while an
`AllocationDef` subject is accepted; (d) `render_mermaid` yields a `flowchart LR` with one
`subgraph` per lane and `-.->` allocate edges, `render_svg` succeeds with `swimlane` groups,
`8,4`-dashed `«allocate»` edges and a dashed unresolved block, and `render_plantuml` yields
nothing.
