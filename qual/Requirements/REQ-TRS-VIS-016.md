---
id: REQ-TRS-VIS-016
type: Requirement
name: The executable lays diagrams out with the same ELK engine as the browser, embedded in-process, so SVG export and docs never need pins or a browser
status: verified
reqDomain: software
verificationMethod: test
---

`syscribe-model` **shall** provide `vis::layout`, which lays out a Diagram IR with the Eclipse
Layout Kernel: the vendored `elk.bundled.js` — the same file the browser bundle uses —
executed in-process by an embedded JavaScript engine (QuickJS), with no Node runtime, browser
or network involved at build or run time. It **shall** apply the same ELK options the browser
client's configurator applies for the diagram's kind (algorithm, direction, hierarchy
handling, port constraints and sides, label placement, spacing, orthogonal routing), honour
pins exactly as the client does (`elk.position` with ELK's interactive layering and ordering
strategies; the `fixed` algorithm when every node is pinned), and return absolute node
rectangles, port positions, label positions and edge routes with bend points, with
hierarchical edge sections translated into root coordinates and reversed-kind edges flipped
back.

`vis::svg` **shall** accept any diagram with an IR: a fully pinned diagram is drawn from its
pins, every other one is laid out by `vis::layout` first. Consequently `syscribe diagram export
--format svg`, `export-html` and MCP `render_diagram format=svg` **shall** succeed on every
diagram that has shapes, and `export-html` **shall** fall back to a companion SVG, a PlantUML
render or a placeholder only when the IR is empty. ELK errors **shall** surface as a
`LayoutError` carrying the JavaScript message, never a panic.

Laying out a diagram of 200 nodes and 300 edges **shall** complete within the test's bound on
the reference developer machine (the product requirement and the hosted test both name 10 s; the test asserts
10 s and prints the measured time), and the vendored bundle **shall** be pinned to the version
the client depends on (`elkjs` in `frontend/package-lock.json`), with a test that fails when
the two diverge.

**Source:** `REQ-TRS-VIS-016` (product model).

**Acceptance criteria:** (a) the derived IBD and BDD of the `vis_derive` fixture lay out with
no sibling overlap, every port centre on its parent's border (±1 px), every edge routed and
every label inside its node; (b) a fully pinned graph uses `fixed` and keeps every pinned
position and size; a partially pinned one carries `elk.position` on the pinned nodes and the
interactive strategies on the root and every compound node; (c) a 200-node / 300-edge BDD lays
out within the bound; (d) `vendor/elkjs/VERSION` equals the `elkjs` version pinned in the
client's lockfile; (e) `diagram export --format svg`, MCP `render_diagram format=svg` and
`export-html` draw an unpinned manifest diagram and refuse only one with no shapes.
