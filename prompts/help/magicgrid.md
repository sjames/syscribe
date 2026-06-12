# magicgrid — MagicGrid B/W/S × 1-4 cell report

```
syscribe -m <root> magicgrid [--audit] [--json]
```

Without `--audit`: bucket every model element by its MagicGrid overlay coordinate
(`custom_fields: { mg_cell: <coord> }`) into the defining rows × pillars grid and
render it. Read-only; available regardless of profile.

With `--audit`: run the gated MagicGrid validation and print a one-screen audit —
the findings, a readiness summary, and a PASS/FAIL verdict (see **Audit** below).

## Coordinates

A coordinate is a **row letter** + **column number**:

- Rows: `B` (problem / black-box), `W` (problem / white-box), `S` (solution).
- Columns: `1` Requirements, `2` Behavior, `3` Structure, `4` Parameters.

The recognised set is `B1`–`B4`, `W1`–`W4`, `S1`–`S4`. An unrecognised coordinate
(or a type/column mismatch) is flagged only under the gated
`validate --profile magicgrid` pass (findings `MG020`/`MG021`); the report itself
is informational.

## Output

The text report prints the full grid, lists and counts the elements classified
into each cell, and marks **empty cells** (a model-completeness hint, not an
error), ending with a count of empty cells.

When exactly one element is marked `custom_fields: { mg_soi: true }` — the System
of Interest, the single black-box (B3) system block — the report prints a
**`System of interest: <name> (B3)`** line. Zero markers is not an error; more than
one is flagged `MG061` under the gated profile.

`--json` emits the same structure: a `cells` object keyed by coordinate
(e.g. `"B2"`), the row/column axes, the empty-cell count, and a
`systemOfInterest` field (the SoI element, or null).

## Audit (`--audit`)

`magicgrid --audit` runs validation with the MagicGrid gate active (as if
`--profile magicgrid`) and rolls the result into a dashboard:

- **Findings** — error/warning counts, a per-code table grouped by category
  (Grid · Refines · Context · SoI · MoE · MoP · Layer · Variant · Coverage), then
  the individual error and warning lines (file + message).
- **Readiness** — grid completeness (populated/empty cells), the System of
  Interest (unique / none / ambiguous), and MoE / MoP / Configuration counts.
- **Verdict** — `PASS` (exit 0) or `FAIL` (exit 2). It FAILs when the gate would
  fail: any MagicGrid error, or a promoted `W307`. The **Coverage** warnings
  (`MG080` orphan need, `MG081` unallocated W2 function, `MG082` missing SoI,
  `MG083` MoE without a MoP) are advisory gap-analysis hints — they appear in the
  listing but do not, by themselves, fail the verdict unless a profile promotes them.

`--json` emits the structured audit (`errors`, `warnings`, `byCode`, `byCategory`,
`findings`, `readiness`, `verdict`, `exitCode`).

## Examples

```
syscribe -m model/ magicgrid
syscribe -m model/ magicgrid --json
syscribe -m model/ magicgrid --audit
syscribe -m model/ magicgrid --audit --json
```

See also: `matrix --allocations`, `trade-study`, `validate --profile magicgrid`.
