---
id: REQ-TRS-MG-015
type: Requirement
title: Tool shall render the magicgrid report as the 3x4 B/W/S × pillar grid
status: draft
reqDomain: software
verificationMethod: test
---

The `magicgrid` report **shall** present the classification as the recognisable **MagicGrid
2-D layout** — three rows (**B** black-box, **W** white-box, **S** solution) by four pillar
columns (**1** Requirements, **2** Behaviour, **3** Structure, **4** Parameters) — not only
as a flat per-cell list. This is the "grid view" of the method.

### Grid matrix

- The report **shall** render a **3×4 grid matrix** of the twelve cells `B1`…`S4` as a
  Markdown table (rows = B/W/S, columns = the four pillars). Each cell **shall** show its
  **element count**; the **System of Interest** cell (`mg_soi`, B3) **shall** be marked;
  and **empty** cells **shall** be marked (a model-completeness hint, not an error).
- The existing **per-cell detail** (each cell's element names) **shall** be retained below
  the matrix, so no information is lost.
- A boxed/aligned **ASCII** rendering of the same matrix is acceptable for the terminal; a
  Markdown table is preferred for the embedded/site docs. Either way the layout is the 3×4
  grid.

### Consistency and JSON

- The grid's populated/empty cell counts **shall** equal the readiness numbers reported by
  `magicgrid --audit` (one source of truth — `REQ-TRS-MG-013`/`-014`).
- `magicgrid --json` **shall** continue to carry the per-cell element lists; it **may**
  additionally expose a grid summary (per-cell counts, SoI, empty set) but **shall not**
  remove existing fields.

**Source:** user request for nicer MagicGrid visualisation (Tier 1 — render the report *as*
the grid). Builds on the `magicgrid` command (`REQ-TRS-MG-003`) and shares the cell/SoI/
empty data with the audit (`REQ-TRS-MG-013`). Tier 2 ([[REQ-TRS-MG-016]]) adds an SVG view;
Tier 3 ([[REQ-TRS-MG-017]]) gives the companion matrices the same treatment.

**Acceptance criteria:**

- `magicgrid` on `model_mg/` prints a 3×4 grid matrix with B/W/S rows and the four pillar
  columns, each cell showing its count.
- The B3 cell is marked as the System of Interest; empty cells (e.g. `S1`/`S2`/`S4` in
  `model_mg/`) are marked as empty.
- The per-cell element names still appear in the report; `--json` still carries the
  per-cell element lists.
- The grid's populated-cell count matches `magicgrid --audit` readiness.
