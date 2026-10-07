---
id: REQ-TRS-VIS-001
type: Requirement
name: One Diagram IR in syscribe-model is the sole input of every renderer and exporter
status: verified
reqDomain: software
verificationMethod: test
---

`syscribe-model` **shall** expose a serialisable Diagram intermediate representation
(`vis::ir`): a `DiagramGraph` with a closed `DiagramKind`, an optional subject, a flat list of
nodes — each with a diagram-local id, the author's element reference and whether it resolved, a
closed `NodeKind`, a display label, an optional stereotype, an optional `parent` node id,
optional port direction and side, compartment lines, an abstract flag and an optional pinned
rectangle — and a list of edges — each with an id, an optional element reference, source and
target node ids, a closed `EdgeKind`, an optional label and optional pinned waypoints — plus
per-kind layout hints.

Every diagram consumer — the sprotty model endpoint, the PlantUML writer and `export-html` —
**shall** take the IR as its only diagram input; no consumer **shall** read `shapes:`, `edges:`
or `layout:` frontmatter directly. An unresolved element reference **shall** be carried in the
IR (drawn, never dropped) with the author's text as its label; the `W402` finding stays the
validator's job.

**Source:** `REQ-TRS-VIS-001` (product model).

**Acceptance criteria:** building the IR of a `Diagram` element written in the string
shorthand and in the map form yields one `Block` node per shape with the resolved element's
type and stereotype, the element's `name` as the label and the `isAbstract` flag carried; a
`parent:` yields IR nesting and a `port`-kind shape becomes a `Port` node of its parent with
its `direction:`; a `layout:` entry becomes the node's pin (and an edge's `points` its
waypoints); the IR round-trips through serde JSON unchanged; and the PlantUML output for every
`pumlMode: companion` diagram of the demo model, rendered from the IR, matches its committed
snapshot.
