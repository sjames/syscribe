---
id: REQ-TRS-MG-004
type: Requirement
name: MagicGrid gate shall validate Measures of Effectiveness (mg_moe)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support **Measures of Effectiveness** (MagicGrid B4) as an overlay on an
existing calculation element, and **shall** validate them under the MagicGrid gate
(`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]). An MoE is **not a new element type**: it is a
`CalculationDef` or `AnalysisCase` marked `custom_fields: { mg_moe: true }`, whose
computation stays in the host element's own `expression`/parameters while flat `mg_moe_*`
custom fields add the stakeholder-yardstick metadata (W041 forbids nested maps, so the
fields are flat). The fields are inert in the base format; the checks below emit `MG03#`
findings only when the gate is active.

### `mg_moe_*` fields

| Field | Meaning |
|---|---|
| `mg_moe` (bool) | marks the host element as an MoE |
| `mg_moe_measures` | the stakeholder need (B1) or use-case objective the MoE gauges |
| `mg_moe_unit` | unit of the measured quantity |
| `mg_moe_direction` | `maximize` or `minimize` (direction of goodness) |
| `mg_moe_threshold` | minimum acceptable value (must-have) |
| `mg_moe_objective` | desired target value (want) |
| `mg_moe_weight` | optional relative weight for a weighted trade study |

### Gate checks (active only under `--profile magicgrid`)

- **`MG030` — wrong host.** `mg_moe: true` on an element that is not a `CalculationDef` or
  `AnalysisCase`.
- **`MG031` — measures missing/unresolved.** `mg_moe_measures` absent, or not resolving
  (by qname or id, §11.10) to a `Requirement`/`RequirementDef` or a use-case objective.
- **`MG032` — bad direction.** `mg_moe_direction` absent or not `maximize`/`minimize`.
- **`MG033` — bad bounds.** `mg_moe_threshold` or `mg_moe_objective` not numeric, or
  inconsistent with the direction (for `maximize`, `objective ≥ threshold`; for `minimize`,
  `objective ≤ threshold`); `mg_moe_weight`, if present, not numeric or outside `[0, 1]`.

**Source:** MagicGrid Measurements of Effectiveness (Parameters pillar, black box, B4).
Overlay design: MoE marked on an existing calculation element, validation gated via
[[REQ-TRS-OUT-012]]. Pairs with the B1 needs it measures and the `mg_cell: B4`
classification of [[REQ-TRS-MG-003]].

**Acceptance criteria:**

- A `CalculationDef` with `mg_moe: true`, `mg_moe_measures: REQ-UAV-ENDUR-001`,
  `mg_moe_unit: km`, `mg_moe_direction: maximize`, `mg_moe_threshold: 10`,
  `mg_moe_objective: 25` validates clean under the gate and raises nothing in the base
  format.
- `mg_moe: true` on a `PartDef` raises `MG030`; a missing/unresolved `mg_moe_measures`
  raises `MG031`.
- `mg_moe_direction: bigger` raises `MG032`; `maximize` with `objective: 5`,
  `threshold: 10` raises `MG033`.
