# Modeling with MagicGrid in Syscribe

**DISCLAIMER:** Syscribe is provided "as-is" without warranty. Output must be independently
reviewed by qualified engineers before any certification or safety use.

You are authoring a **MagicGrid** systems model in the **Syscribe** format. This prompt
teaches the MagicGrid method and exactly how to express it with this tool. It **assumes the
base Syscribe format** — run `syscribe --agent-instructions` (no topic) for that: every model
element is a `.md` file with YAML frontmatter under a directory tree; the directory path is the
namespace; qualified names are `::`-separated and **relative to the model root** (the root
package `name:` is **not** a segment — write `Problem::BlackBox::Foo`, never `<Root>::Problem::…`).

MagicGrid is a pure **overlay**: all MagicGrid-specific data rides on `custom_fields:` keys
prefixed **`mg_`** (flat scalars only — no nested maps), plus the base `refines:`, `actors:`,
and `allocatedTo:` fields. **Nothing changes in the base format**, and all MagicGrid-specific
validation is gated behind a profile, so a non-MagicGrid model is unaffected.

---

## 1. The grid

MagicGrid organises the model as a matrix: **rows** are abstraction domains, **columns** are
the four SysML pillars. The build flows top-left to bottom-right:

```
                 1 Requirements   2 Behavior        3 Structure          4 Parameters
Problem · Black   B1 stakeholder   B2 use cases      B3 system context    B4 Measures of
  box (B)            needs            (actors)          (SoI + externals)     Effectiveness (MoE)
Problem · White   W1 system        W2 functional     W3 logical           W4 Measures of
  box (W)            requirements     analysis          subsystems            Performance (MoP)
Solution (S)      (component reqs) (component beh.)  S3 physical          S4 design
                                                       components            parameters
```

Method flow: **stakeholder needs → use cases → system context → MoEs → system requirements →
functional analysis → logical subsystems → physical components**, with measures (MoE→MoP) and
allocations tying the cells together.

## 2. Turn on the gate

Declare the profile once in `<model_root>/.syscribe.toml`:

```toml
[profiles.magicgrid]
magicgrid = true
promote = ["W307"]   # treat "use case with no refines" as a gate failure
```

Then validate with `syscribe -m <root> validate --profile magicgrid`. The `MG###` checks fire
**only** under this profile. The reports below are read-only and work regardless of profile.

## 3. Field reference (all on `custom_fields:` unless noted)

| Field | On | Meaning |
|---|---|---|
| `mg_cell: <coord>` | any element | grid coordinate `B1`–`B4`/`W1`–`W4`/`S1`–`S4`; the pillar (column) must match the element type |
| `refines: [REQ-…]` | **base field** on `UseCaseDef`/`ActionDef`/`StateDef` | use case / function refines (gives behaviour to) a requirement; `refinedBy` is derived |
| `actors: [Part…]` | **base field** on `UseCaseDef` | external actor parts of a use case |
| `mg_external: true` | `Part`/`PartDef` | element is outside the system-of-interest boundary (B3) |
| `mg_soi: true` | `Part`/`PartDef` | the system of interest (exactly one; not also `mg_external`) |
| `mg_moe: true` + `mg_moe_measures`/`_unit`/`_direction`(`maximize`\|`minimize`)/`_threshold`/`_objective`/`_weight` | `CalculationDef`/`AnalysisCase` | a Measure of Effectiveness (B4) gauging a stakeholder need |
| `mg_mop: true` + `mg_mop_refines: <MoE>` + `mg_mop_unit` | `CalculationDef`/`ConstraintDef`/`AnalysisCase` | a Measure of Performance (W4/S4) refining a B4 MoE; `mopRefinedBy` is derived |
| `mg_layer: logical` \| `physical` | `Part`/`PartDef` | W3 logical subsystem vs S3 physical component |
| `allocatedTo: <target>` | **base field** on the source element | allocation: function→logical, logical→physical; `allocatedFrom` is derived |
| `mg_variant: true` | `Configuration` | parametric variant — may omit `featureModel:`; scored by `trade-study` from its `parameterBindings:` |

## 4. Authoring workflow (cell by cell)

**B1 — Stakeholder needs.** `type: Requirement`, `requirementKind: stakeholder`, a stable
`REQ-*` id, `custom_fields: { mg_cell: B1 }`. Each need must be refined by a use case or
derived into a system requirement (else `MG080`).

**B2 — Use cases.** `type: UseCaseDef`, `mg_cell: B2`, `refines: [<B1 need>]` (every non-draft
use case must refine something → `W307`), `actors: [<external parts>]` (each actor must be a
`Part` marked `mg_external: true` → `MG010`–`MG013`).

**B3 — System context.** One `PartDef` for the system of interest marked
`custom_fields: { mg_soi: true, mg_cell: B3 }`; the external actors/systems are `PartDef`s
marked `mg_external: true`. (`MG060`–`MG062`, `MG082`.)

**B4 — Measures of Effectiveness.** A `CalculationDef` marked `mg_moe: true` with an
`expression:` and the `mg_moe_*` fields, `mg_cell: B4`, `mg_moe_measures:` a B1 need.
(`MG030`–`MG033`.)

**W1 — System requirements.** `type: Requirement`, `derivedFrom: [<B1 need>]`,
`breakdownAdr: <accepted ADR>`, `mg_cell: W1`.

**W2 — Functional analysis.** `ActionDef`/`StateDef`, `mg_cell: W2`, `refines: [<W1 req>]`,
and `allocatedTo: <W3 logical subsystem>` (a W2 function allocated to no logical part →
`MG081`).

**W3 — Logical subsystems.** `PartDef`, `custom_fields: { mg_layer: logical, mg_cell: W3 }`,
`allocatedTo: <S3 physical component>` (an unrealised logical part → `MG041`). Do **not** link
logical↔physical via `supertype:`/`typedBy:` (→ `MG042`).

**W4 — Measures of Performance.** A `ConstraintDef`/`CalculationDef` marked `mg_mop: true`,
`mg_mop_refines: <a B4 MoE>`, `mg_cell: W4`. Every MoE should have a MoP (else `MG083`).

**S3 — Physical components.** `PartDef`, `custom_fields: { mg_layer: physical, mg_cell: S3 }`.

**Configurations & trade study.** Add ≥2 `Configuration`s with `parameterBindings:` (numeric
map). For parameter-only variants set `custom_fields: { mg_variant: true }` and omit
`featureModel:`. `trade-study` scores them against the MoEs.

## 5. Allocation — two forms

Default (lightweight, OSLC): put `allocatedTo: <target>` on the **source** element; the reverse
`allocatedFrom` is **derived** (`show` prints it under `## Allocated from`). Use a standalone
`type: Allocation` element (naming both `allocatedFrom` and `allocatedTo`) **only** when the
allocation needs a documented body (rationale, freedom-from-interference argument). Declaring
the same edge in both forms is redundant (`W503`).

## 6. Validate & inspect

```bash
syscribe -m <root> validate --profile magicgrid   # the gate — drive it clean
syscribe -m <root> magicgrid                       # the B/W/S × 1-4 grid + the System of Interest
syscribe -m <root> magicgrid --audit               # findings rollup + readiness + PASS/FAIL verdict
syscribe -m <root> trade-study                      # MoE-weighted scoring of every Configuration
syscribe -m <root> matrix --allocations            # function→structure / logical→physical matrix
syscribe -m <root> show <element>                   # see derived refinedBy / allocatedFrom / mopRefinedBy
```

Target: `magicgrid --audit` reports **`Verdict: PASS`** with zero findings.

## 7. Finding codes — what to fix

| Code | Fix |
|---|---|
| `E316` / `W307` | a `refines:` operand is unresolved/not-a-requirement / a non-draft use case has no `refines:` |
| `MG010`–`MG013` | an `actors:` entry is unresolved / not a part / not `mg_external` / a use case has no actor |
| `MG020`/`MG021` | `mg_cell` is not a valid coordinate / the element type doesn't match the column |
| `MG030`–`MG033` | MoE host/`measures`/`direction`/bounds are wrong |
| `MG040`–`MG042` | bad `mg_layer` / logical part not realised / logical↔physical via supertype |
| `MG050`–`MG052` | MoP host / `mg_mop_refines` missing / target is not an MoE |
| `MG060`–`MG062` | `mg_soi` host / more than one SoI / SoI also external |
| `MG070` | `mg_variant` on a non-`Configuration` |
| `MG080`–`MG083` (coverage) | orphan need / unallocated W2 function / no SoI / MoE without a MoP |

## 8. Worked micro-example (a smart thermostat)

```
# Problem/BlackBox/Needs/ComfortNeed.md
type: Requirement
id: REQ-TH-NEED-001
name: The occupant shall stay comfortable
status: approved
reqDomain: system
custom_fields: { mg_cell: B1 }
---
# Problem/BlackBox/UseCases/MaintainTemperature.md
type: UseCaseDef
name: MaintainTemperature
status: approved
refines: [REQ-TH-NEED-001]
actors: [Occupant]
custom_fields: { mg_cell: B2 }
---
# Problem/BlackBox/Context/Thermostat.md   (the SoI)
type: PartDef
name: Thermostat
custom_fields: { mg_soi: true, mg_cell: B3 }
---
# Problem/BlackBox/Context/Occupant.md     (external actor)
type: PartDef
name: Occupant
custom_fields: { mg_external: true, mg_cell: B3 }
---
# Problem/WhiteBox/Functions/RegulateTemp.md
type: ActionDef
name: RegulateTemp
allocatedTo: Problem::WhiteBox::Logical::ControlLoop
custom_fields: { mg_cell: W2 }
---
# Problem/WhiteBox/Logical/ControlLoop.md
type: PartDef
name: ControlLoop
allocatedTo: Solution::Physical::McuBoard
custom_fields: { mg_layer: logical, mg_cell: W3 }
---
# Solution/Physical/McuBoard.md
type: PartDef
name: McuBoard
custom_fields: { mg_layer: physical, mg_cell: S3 }
```

## 9. Definition of done

- `validate --profile magicgrid` → exit 0, no `MG###` errors.
- `magicgrid --audit` → `Verdict: PASS`.
- Every B1 need traces down; every use case refines a need and names an external actor; the SoI
  is set once; every MoE has a MoP; every W2 function and every logical part is allocated.
- A complete worked reference model lives in **`model_mg/`** (an EV DC fast-charging station) —
  read it with `syscribe -m model_mg magicgrid --audit`, `… trade-study`, and `… show <element>`.
