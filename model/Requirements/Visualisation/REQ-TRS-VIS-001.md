---
type: Requirement
id: REQ-TRS-VIS-001
name: "One Diagram IR in syscribe-model is the sole input of every renderer and exporter"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

`syscribe-model` shall expose a serialisable Diagram intermediate representation (`vis::ir`)
consisting of a `DiagramGraph` with a closed `DiagramKind`, an optional subject, a flat list of
nodes (each with a diagram-local id, an optional resolved element reference, a closed
`NodeKind`, a display label, an optional stereotype, an optional `parent` node id, optional port
direction and side, compartment lines, an abstract flag and an optional pinned rectangle) and a
list of edges (each with an id, an optional element reference, source and target node ids, a
closed `EdgeKind`, an optional label and optional pinned waypoints), plus per-kind layout
hints.

Every diagram consumer — the sprotty model endpoint, the PlantUML writer, the Mermaid writer,
the SVG writer and `export-html` — shall take the IR as its only diagram input. No consumer
shall read `shapes:`, `edges:` or `layout:` frontmatter directly.

## Rationale

Every defect the 2026-10-07 review found traces to the same manifest being parsed by different
code with different gaps. One value type with closed vocabularies makes each backend a pure
function that can be snapshot-tested, and makes "is this diagram well-formed" a single question.

## Scope

- The IR is a value type: no sprotty, PlantUML or SVG knowledge inside it.
- Unresolved element references are carried as `None` references with the author's text as the
  label (drawn dashed), never dropped; the `W402` finding is the validator's job.
- Serialisation is `serde` JSON, so the sprotty writer, tests and the MCP tools share one shape.
