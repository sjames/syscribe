---
id: REQ-TRS-MG-003
type: Requirement
title: MagicGrid gate shall classify elements by grid cell and render a grid report
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** let a model place each element on the MagicGrid via a
`custom_fields: { mg_cell: <coord> }` overlay field, **shall** validate those coordinates
under the MagicGrid gate (`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]), and **shall** render
the grid as a report. The `mg_cell` value is inert data in the base format (queryable via
`validate --where`); the checks below fire only when the gate is active and emit `MG02#`
findings.

### Grid coordinates

A coordinate is a **row letter** + **column number**: row ∈ `B` (problem/black-box),
`W` (problem/white-box), `S` (solution); column ∈ `1` Requirements, `2` Behavior,
`3` Structure, `4` Parameters. The recognised set is therefore `B1`–`B4`, `W1`–`W4`,
`S1`–`S4` (e.g. `B1` stakeholder needs, `B2` use cases, `B3` system context, `B4` MoEs).

### Gate checks (active only under `--profile magicgrid`)

- **`MG020` — invalid coordinate.** `mg_cell` is not one of the recognised coordinates.
- **`MG021` — type/column mismatch.** The element's `type` is incompatible with the pillar
  implied by the column number, under this mapping:
  - col `1` Requirements → `Requirement`/`RequirementDef`
  - col `2` Behavior → `UseCaseDef`/`UseCase`/`ActionDef`/`Action`/`StateDef`/`State`
  - col `3` Structure → `Part`/`PartDef`/`Port`/`PortDef`/`Interface`/`InterfaceDef`/`Connection`/`ConnectionDef`
  - col `4` Parameters → `ConstraintDef`/`Constraint`/`CalculationDef`/`Calculation`/`AnalysisCase`

### Grid report

- `syscribe -m <root> magicgrid [--json]` **shall** render the rows × pillars grid, listing
  (and counting) the elements classified into each cell and **flagging empty cells**
  (informational — an empty cell is a modelling-completeness hint, not an error). The
  command is read-only and available regardless of profile; `--json` emits the grid
  structure.

**Source:** MagicGrid grid structure (the defining rows × pillars matrix). Overlay design:
classification on `mg_cell`, validation gated via [[REQ-TRS-OUT-012]]. Related:
[[REQ-TRS-MG-001]], [[REQ-TRS-MG-002]], [[REQ-TRS-MG-004]].

**Acceptance criteria:**

- An element with `custom_fields: { mg_cell: B2 }` is reported in the Black-box × Behavior
  cell by `magicgrid`; with the gate inactive it raises no finding.
- Under `validate --profile magicgrid`, `mg_cell: X9` raises `MG020`, and a `PartDef` with
  `mg_cell: B1` (Requirements column) raises `MG021` naming the type/column conflict.
- `magicgrid` prints the full `B/W/S × 1–4` grid with per-cell element counts and marks
  cells that contain no element; `magicgrid --json` emits the same structure.
