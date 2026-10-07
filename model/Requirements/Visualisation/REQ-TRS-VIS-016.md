---
type: Requirement
id: REQ-TRS-VIS-016
name: "The executable lays diagrams out with the same ELK engine as the browser, embedded in-process, so SVG export and docs never need pins or a browser"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - elk
---

`syscribe-model` shall provide `vis::layout`, which lays out a Diagram IR with the Eclipse Layout
Kernel: the vendored `elk.bundled.js` — the same file the browser bundle uses — executed
in-process by an embedded JavaScript engine (QuickJS), with no Node runtime, browser or network
involved at build or run time. It shall apply the same ELK options the browser client's
configurator applies for the diagram's kind (algorithm, direction, hierarchy handling, port
constraints and sides, label placement, spacing, orthogonal routing), honour pins exactly as the
client does (interactive layering with pinned positions; the `fixed` algorithm when every node
is pinned), and return absolute node rectangles, port positions and edge routes with bend
points.

`vis::svg` shall accept any diagram with an IR: fully pinned diagrams are drawn from their pins,
all others are laid out by `vis::layout` first. Consequently `syscribe diagram export --format
svg`, `export-html` and MCP `render_diagram format=svg` shall succeed on every diagram that has an
IR, and `export-html` shall fall back to a companion SVG or placeholder only when the IR is empty.

Laying out a diagram of 200 nodes and 300 edges shall complete within 2 s on the reference
developer machine, and the vendored bundle shall be pinned to the version the client depends on
(`elkjs` in `frontend/package.json`), with a test that fails when the two diverge.

## Rationale

The user chose ELK for its quality and wants that quality in every output, not only in the
browser. Embedding the engine is the only option that keeps the project's "no runtime Node"
posture; the 2026-10-07 spike proved it works in under half a second cold.

## Scope

- `rquickjs` (QuickJS, MIT) becomes a dependency of `syscribe-model`; a C compiler is needed at
  build time. The bundle and its EPL-2.0 licence are committed under `crates/syscribe-model/vendor/`.
- Edge routing output is consumed by `vis::svg` as waypoints; the browser is unaffected by this
  requirement except through `REQ-TRS-VIS-017`.
