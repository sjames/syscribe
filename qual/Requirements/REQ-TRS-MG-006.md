---
id: REQ-TRS-MG-006
type: Requirement
name: Tool shall render an allocation matrix view
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** render allocation coverage as a matrix, the table MagicGrid reviewers
use to read function→structure and logical→physical allocation. This is a **read-only
reporting** feature (like the existing Requirement × Configuration `matrix`), available
regardless of profile; it is MagicGrid-aware in that it understands the `mg_layer` overlay
([[REQ-TRS-MG-005]]) when present, but introduces no gate of its own.

### Command

- `syscribe -m <root> matrix --allocations [--json]` **shall** build a matrix from the
  model's `Allocation` elements (and `allocatedFrom:`/`allocatedTo:` links): **rows** are
  allocation sources (e.g. behaviour actions / logical parts), **columns** are allocation
  targets (e.g. structural / physical parts), and each **cell** marks an allocation present
  (`✓`) or a gap. A per-row/per-column rollup **shall** report unallocated sources and
  unused targets.
- When `mg_layer` is present in the model, the view **shall** be partitionable into the
  **logical→physical** allocation matrix (sources `mg_layer: logical`, targets
  `mg_layer: physical`); absent any `mg_layer`, it falls back to the flat source/target
  view derived purely from `Allocation` elements.
- `--json` **shall** emit the grid structure (rows, columns, cells, and the rollup).

**Source:** MagicGrid allocation matrices (function↔structure, logical↔physical), the
review artifact the existing `Allocation` summary (§8 of the validation report) does not
provide. Parallel to the Requirement × Configuration `matrix` command; consumes the
layering of [[REQ-TRS-MG-005]].

**Acceptance criteria:**

- `matrix --allocations` lists every `Allocation` source as a row and every target as a
  column, marks allocated cells, and reports any source with no allocation and any target
  never allocated to.
- In a model using `mg_layer`, the view separates logical sources from physical targets;
  in a model without it, the flat `Allocation`-derived matrix is produced.
- `matrix --allocations --json` emits the same grid as structured JSON.
