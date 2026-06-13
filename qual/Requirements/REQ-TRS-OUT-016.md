---
id: REQ-TRS-OUT-016
type: Requirement
name: Tool shall generate an N² interface matrix (n2 command)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only **`n2`** command (§16, GH #64) that generates an N²
(N-squared) interface matrix from the model's existing port / connection / flow / allocation
edges — no new element types or validation rules.

**`syscribe n2 [<qname>] [--depth <N>] [--format text|html|json] [--interfaces-only] [--allocations]`**

- **Axis.** Without `<qname>`, every `PartDef`/`Part` in the model is an axis element. With
  `<qname>`, the axis is the composite element's subpart types (features typed by a
  `PartDef`/`Part`), expanded `--depth` levels (default 1).
- **Cells.** The diagonal holds the element; the off-diagonal cell at `(row R, col C)`
  **shall** list the interfaces directed from `R` to `C` — each a named interface taken from
  the connection's `typedBy:` (the ConnectionDef/InterfaceDef, last segment) or its `name`.
  Edge kinds: **Connection** (`connections:`), **Flow** (`flowConnections:`), and — with
  `--allocations` — **Allocation** (`allocatedTo:`). Endpoint feature chains resolve through
  the owning element's `features:`/`typedBy:` (the canonical SysML port wiring).
- **Formats.** `text` (ASCII grid), `html` (a self-contained `<table>` fragment), and `json`
  (`{ scope, elements, matrix: { R: { C: [{kind, name}] } } }`).
- **`--interfaces-only`** **shall** drop axis elements that have no interfaces.

**Source:** §16 (N² Interface Matrix), GH #64. Port interfaces are surfaced via the
connections/flows that bind their (conjugate) ports — the explicit SysML wiring.

**Acceptance criteria:**

- `n2 <composite>` produces a square matrix whose off-diagonal cells name the connecting
  interfaces; the diagonal is the element marker.
- Connection and Flow edges appear; `--allocations` adds allocation edges.
- `--depth` controls how far the subpart axis expands.
- `--interfaces-only` removes elements with no interfaces.
- `--format json` matches the `{ scope, elements, matrix }` schema; `--format html` is a
  valid self-contained table.
