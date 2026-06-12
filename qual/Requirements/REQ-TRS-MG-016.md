---
id: REQ-TRS-MG-016
type: Requirement
title: Tool shall render the MagicGrid as an SVG via magicgrid --svg, usable as a Diagram companion
status: draft
reqDomain: software
verificationMethod: test
---

The `magicgrid` command **shall** offer a **`--svg`** mode that renders the 3×4 B/W/S ×
pillar grid as a standalone **SVG** drawing — a publishable visualisation that can be
attached as the **companion SVG of a `Diagram` element** (Tier 2).

### Output

- `syscribe -m <root> magicgrid --svg` **shall** emit a complete, self-contained SVG
  document to **stdout**; `-o <file>` **shall** write it to that file instead (mirroring
  `diagram render`/`compose`). No external tools or network are required — the SVG is
  produced deterministically in-process.
- The produced file is intended to be a **`Diagram` companion**: writing it to
  `<Name>.svg` beside a `Diagram` element with `svgMode: companion` (default same-stem
  `.svg`, §diagram rules) satisfies the companion-on-disk check (`E402`) so the MagicGrid
  renders in the browser/detail view like any other diagram.

### Visual content

The SVG **shall** depict:
- the **three rows** B/W/S (visually distinguished, e.g. colour-banded) and the **four
  pillar columns** with headers (Requirements/Behaviour/Structure/Parameters);
- each of the twelve cells as a box containing its elements (names or chips), with a
  per-cell count;
- the **System of Interest** (B3) highlighted, and **empty** cells visually de-emphasised;
- a title/legend identifying it as the MagicGrid for the model.

### Layout and word wrapping

- The grid layout (row/column sizing, cell placement) **shall** be computed with the
  shared diagram layout engine (`taffy`, as used by the element-diagram renderer) so the
  MagicGrid is consistent with the other diagrams.
- A cell element label that is wider than its cell **shall** be **word-wrapped** across
  multiple lines (broken on word boundaries, measured with the shared font metrics) and
  the cell/row **shall** grow to fit the wrapped lines — labels **shall not** be silently
  truncated. A single word longer than the cell width may be hard-broken.

### Consistency

- The SVG's cells, SoI, and empty set **shall** be derived from the **same** `mg_cell`/
  `mg_soi` data as the text report ([[REQ-TRS-MG-015]]) and the audit — they can never
  disagree.

**Source:** user request — "the SVG should be a `magicgrid --svg` that we can put as a
companion to a Diagram markdown." Companion of the Tier 1 text grid ([[REQ-TRS-MG-015]]).

**Acceptance criteria:**

- `magicgrid --svg` on `model_mg/` prints a well-formed `<svg …>…</svg>` document to
  stdout (a single root `<svg>`), with the three row labels (B/W/S) and the four pillar
  column headers present.
- `magicgrid --svg -o <file>` writes the same SVG to `<file>` and prints nothing to
  stdout; the file parses as XML with one `<svg>` root.
- The SVG marks the B3 System of Interest and renders the populated cells of `model_mg/`.
- Writing the SVG as a `Diagram` element's same-stem companion (`svgMode: companion`)
  validates with no `E402` (companion exists on disk).
- A cell whose element label is wider than the cell is word-wrapped onto multiple lines
  (every word of the label is present in the SVG, no `…` truncation), and the SVG height
  grows to accommodate the wrapped lines.
