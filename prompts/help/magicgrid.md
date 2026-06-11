# magicgrid — MagicGrid B/W/S × 1-4 cell report

```
syscribe -m <root> magicgrid [--json]
```

Bucket every model element by its MagicGrid overlay coordinate
(`custom_fields: { mg_cell: <coord> }`) into the defining rows × pillars grid and
render it. Read-only; available regardless of profile.

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

## Examples

```
syscribe -m model/ magicgrid
syscribe -m model/ magicgrid --json
```

See also: `matrix --allocations`, `trade-study`.
