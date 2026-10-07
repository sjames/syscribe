---
type: Requirement
id: REQ-TRS-VIS-010
name: "A static SVG writer draws any diagram per §8.16.5 — from its pins when fully pinned, else laid out by the embedded ELK — and export-html falls back to a companion only when there is nothing to draw"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - export
---

`vis::svg` shall draw any diagram that has an IR with at least one node, emitting SVG that
conforms to spec §8.16.5: the `sysml:` namespace, `sysml:ref` on every shape and
`sysml:ref`/`sysml:source`/`sysml:target` on every edge, the element-kind CSS classes and the
shared visual language of `REQ-TRS-VIS-012`. A fully pinned IR is drawn from its pins (sizes from
the pins' `w`/`h`, else from the shared metrics of `REQ-TRS-VIS-017`); any other IR is laid out
first by the embedded engine of `REQ-TRS-VIS-016`, using its node rectangles, port positions and
edge polylines verbatim. The writer shall fail only for an IR with no nodes.

`export-html` shall embed a diagram by the first applicable rule: (1) the IR has nodes, draw it
with `vis::svg`; (2) the diagram has a companion SVG (`svgMode: companion`/`svgFile:`), embed that
file; (3) a PlantUML-rendered `.svg` exists beside the companion `.puml`, embed it; (4) otherwise
emit a placeholder showing the diagram's name, kind and subject.

## Rationale

With the same ELK engine inside the executable (`ADR-SYS-VIS-001`, addendum "the same ELK engine
inside the executable"), the picture in a document no longer depends on a browser visit or on
pins; pins and saved companions remain the way to freeze a hand-adjusted layout.

## Scope

- This supersedes the original pinned-only rule and the four-step fallback chain of the first
  cut (`PI-VIS-004`); `PI-VIS-006` delivered the change.
- `[links]`-driven hyperlink wrapping of shapes (`REQ-TRS-LINK-002`) applies to the SVG the
  writer emits.
