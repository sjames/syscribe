---
type: PlanningItem
id: PI-VIS-006
name: "Phase 3b — ELK embedded in the executable (QuickJS + vendored elkjs), Rust-owned node sizes shared with the browser, SVG export and docs without pins"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-016, REQ-TRS-VIS-017]
tags:
  - visualisation
---

`vis::layout` (elk.bundled.js under `rquickjs`), `vis::metrics` (promoted from `svgkit`), sizes
carried in the sprotty graph and consumed by the client, `vis::svg` laying out unpinned diagrams,
`export-html`/`diagram export`/MCP on every diagram, the Node-vs-QuickJS determinism test and the
bundle-version pin test. ADR: `Decisions::VisualisationADR`, addendum "the same ELK engine inside
the executable". Spike evidence: 2026-10-07, ~0.4 s cold layout of an IBD-shaped graph.
