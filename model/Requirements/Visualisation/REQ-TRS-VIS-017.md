---
type: Requirement
id: REQ-TRS-VIS-017
name: "Node sizes are computed once in Rust from shared text metrics and used by both renderers, so the browser and the executable produce the same layout"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

`syscribe-model` shall own the text metrics used to size diagram content (`vis::metrics`,
promoted from the `svgkit` metrics the MagicGrid report already uses: system-font measurement
with a bundled approximate fallback), and shall compute from them the size of every node, port,
label and compartment of an IR — name, stereotype and banner lines, compartment lines, port
label — with a documented safety margin. The sprotty graph (`REQ-TRS-VIS-006`) shall carry that
`size` on every element, and the browser client shall use the carried sizes instead of
measuring text in the DOM. The client's CSS font stack and the metrics' font family shall be the
same, and the margin shall be such that a label never overflows its box in the browser.

Given the same diagram, pins and options, the coordinates produced by the browser's ELK and by
`vis::layout` shall be identical, verified by a test that lays out a fixture graph with
`vis::layout` and compares it to a committed result produced by `elkjs` under Node from the same
ELK input (the `npm test` script exports that input and result).

## Rationale

ELK is deterministic: identical input yields identical output. Making Rust the single source of
node sizes is what turns "the same engine on both sides" into literally the same picture in the
browser, in `diagram export`, in `export-html` and in a saved companion SVG.

## Scope

- Sprotty's micro-layout (`vbox`) remains in use for positioning children inside a node; only
  the measured sizes are replaced by the carried ones.
- Fonts differ between machines; the metrics fall back to the approximate table when no system
  font is found, and the margin absorbs the difference. Pixel identity across machines is not
  required; identity between the two renderers on one machine is.
