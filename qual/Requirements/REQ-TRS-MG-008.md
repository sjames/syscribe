---
id: REQ-TRS-MG-008
type: Requirement
title: MagicGrid gate shall validate Measurements of Performance (mg_mop) and their refinement of MoEs
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support **Measurements of Performance** (MoP) as a first-class MagicGrid
concept distinct from the black-box Measures of Effectiveness of [[REQ-TRS-MG-004]], and
**shall** validate them under the MagicGrid gate (`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]).
MagicGrid V2 separates **MoEs** (black-box, stakeholder-facing, cell B4) from **MoPs**
(white-box / solution engineering measures, cells W4 / S4) that **refine** the MoEs they
support; today only `mg_moe` exists, so the MoE→MoP measurement chain cannot be expressed
(the gap surfaced while modelling `model_mg/`, whose W4 cell held untyped `ConstraintDef`s
with no trace to its B4 MoEs).

An MoP is **not a new element type**: it is a `CalculationDef`, `ConstraintDef`, or
`AnalysisCase` marked `custom_fields: { mg_mop: true }`, typically placed in cell W4/S4
(`mg_cell`, [[REQ-TRS-MG-003]]). Its data rides on flat `mg_mop_*` custom fields (W041:
scalars only). The fields are inert in the base format; the checks below emit `MG05#`
findings only when the gate is active.

### `mg_mop_*` fields

| Field | Meaning |
|---|---|
| `mg_mop` (bool) | marks the host element as a Measurement of Performance |
| `mg_mop_refines` | the black-box MoE (an `mg_moe` element) this performance measure refines/supports |
| `mg_mop_unit` | unit of the measured quantity (optional) |

### Gate checks (active only under `--profile magicgrid`)

- **`MG050` — wrong host.** `mg_mop: true` on an element that is not a `CalculationDef`,
  `ConstraintDef`, or `AnalysisCase`.
- **`MG051` — refinement missing/unresolved.** `mg_mop_refines` absent, or not resolving
  (by qname or id, §11.10) to a model element.
- **`MG052` — refinement target is not an MoE.** `mg_mop_refines` resolves to an element
  that is not marked `mg_moe: true` (a MoP must refine an MoE, not an arbitrary element).

The tool **shall** also compute the inverse index **`mopRefinedBy`** on each MoE (the MoPs
that refine it), surfaced in `show` like `refinedBy`/`actorIn`, so the MoE→MoP measurement
chain is navigable from the MoE.

**Source:** MagicGrid Measurements of Performance (W4/S4) refining black-box MoEs (B4) —
gap identified building the `model_mg/` EV-charging-station model. Overlay design: MoP
marked on an existing calculation/constraint element, validation gated via
[[REQ-TRS-OUT-012]]; pairs with the MoEs of [[REQ-TRS-MG-004]] and the trade study of
[[REQ-TRS-MG-007]].

**Acceptance criteria:**

- A `ConstraintDef` with `mg_mop: true` and `mg_mop_refines:` naming an `mg_moe`
  `CalculationDef` validates clean under the gate, and the referenced MoE reports the
  constraint under its computed `mopRefinedBy` index.
- `mg_mop: true` on a `PartDef` raises `MG050`.
- A missing or unresolved `mg_mop_refines` raises `MG051`; one resolving to a non-MoE
  element (e.g. a plain `CalculationDef` without `mg_moe`) raises `MG052`.
- All `mg_mop_*` fields are inert (no finding) when the magicgrid profile is not active.
