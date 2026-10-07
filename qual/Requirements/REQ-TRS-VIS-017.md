---
id: REQ-TRS-VIS-017
type: Requirement
name: Node sizes are computed once in Rust from shared text metrics and used by both renderers, so the browser and the executable produce the same layout
status: verified
reqDomain: software
verificationMethod: test
---

`syscribe-model` **shall** own the text metrics used to size diagram content (`vis::metrics`,
promoted from the `svgkit` metrics the MagicGrid report already uses: system-font measurement
over the Helvetica → Arial → Liberation Sans → DejaVu Sans → any-sans stack, with a bundled
approximate fallback), and **shall** compute from them (`vis::size`) the size of every node,
port, label and compartment of an IR — name, stereotype and banner lines, compartment lines,
port label, edge keyword and label — following the browser client's own stacking recipe
(`vbox` paddings and gaps, 12×12 ports, the client's minimum node box, room for ports on the
sides they use) with a documented safety margin (8 % plus 2 px of width, a 1.25 line factor).
The sprotty graph (`REQ-TRS-VIS-006`) **shall** carry that `size` on every node, port,
compartment and label element, and the embedded ELK (`REQ-TRS-VIS-016`) **shall** build its
input from the same sizes.

Given the same diagram, pins and options, the coordinates produced by `elkjs` under Node and
by `vis::layout` under QuickJS **shall** be identical, verified by a test that runs the
committed ELK input of a fixture graph through the embedded engine and compares the result,
after JSON canonicalisation and with no tolerance, to the committed result `elkjs` produced
under Node from the same input.

**Source:** `REQ-TRS-VIS-017` (product model).

**Acceptance criteria:** (a) a block's label boxes stack as the client's `vbox` does
(stereotype, banners, name, then compartments at their offsets) and a compartment's lines at
theirs; a port is 12×12 with a name label; a leaf is raised to the client's minimum and to
what its ports need; a pin with both `w` and `h` is carried instead of the measurement;
(b) `to_sgraph` emits `size` on every node, port, compartment and label and sized label
children (`role` + `size`) for stereotypes, banners, names, lines and edge keywords/labels;
(c) the ELK input `vis::layout` builds for the fixture IBD equals its committed snapshot, and
the embedded engine's output for it equals the committed Node output exactly;
(d) `load_metrics()` keeps the MagicGrid report's font stack so its SVG is unchanged.
