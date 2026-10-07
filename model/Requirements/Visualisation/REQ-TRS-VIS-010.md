---
type: Requirement
id: REQ-TRS-VIS-010
name: "A static SVG writer draws fully pinned diagrams per §8.16.5, and export-html falls back to a companion SVG, a PlantUML render or a placeholder"
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

`vis::svg` shall draw an IR whose every node has a pin, emitting SVG that conforms to spec
§8.16.5: the `sysml:` namespace, `sysml:ref` on every shape and `sysml:ref`/`sysml:source`/
`sysml:target` on every edge, the element-kind CSS classes and the shared visual language of
`REQ-TRS-VIS-012`. It shall refuse (return `None`) when any node lacks a pin; it never computes
positions.

`export-html` shall embed a diagram by the first applicable rule: (1) the IR is fully pinned,
draw it with `vis::svg`; (2) the diagram has a companion SVG (`svgMode: companion`/`svgFile:`),
embed that file; (3) a PlantUML-rendered `.svg` exists beside the companion `.puml`, embed it;
(4) otherwise emit a placeholder showing the diagram's name, kind and subject and a link that
opens it in the browser.

## Rationale

With layout in the browser, the server cannot produce a picture from nothing; the honest
artefacts for documents are pins a human set or an SVG a human saved. Making the fallback
chain explicit keeps `export-html` deterministic and never silently blank.

## Scope

- The placeholder is a visible, accepted regression for unpinned diagrams in the demo model
  until the browser is used to save them (`ADR-SYS-VIS-001` consequences).
- `[links]`-driven `<a href>` wrapping of shapes keeps working on the SVG the writer emits.
