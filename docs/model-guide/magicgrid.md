# MagicGrid

`GUIDE · MAGICGRID`

[MagicGrid](https://www.3ds.com/products/catia/no-magic) (Dassault / No Magic) is a
method that lays a system model out on a grid: three **rows** — problem black-box
(**B**), problem white-box (**W**), solution (**S**) — crossed with four **pillars**
— Requirements (**1**), Behaviour (**2**), Structure (**3**), Parameters (**4**).
Syscribe supports the method as a thin **overlay**: nothing changes in the base
format, all the data rides on `custom_fields:`, and every MagicGrid-specific
validation is gated behind an opt-in profile. A model with no MagicGrid fields
behaves exactly as before.

The bundled **`model_mg/`** is a complete worked example — a DC fast-charging
station — that you can run every command in this guide against.

---

## 1. The overlay philosophy

MagicGrid in Syscribe is **pure overlay**:

- **All data lives on `custom_fields:`** with the **`mg_` prefix** (plus the base
  `actors:` field, which already exists). These are flat scalars — `custom_fields:`
  never grows a nested map — so they are inert in the base format: no schema field,
  no base-format check fires on them.
- **Nothing is added to the base format.** A `PartDef` with `mg_cell: B3` is still
  an ordinary `PartDef`.
- **All MagicGrid validation is gated** behind a profile. Declare it once in
  `<model_root>/.syscribe.toml`:

```toml
[profiles.magicgrid]
magicgrid = true
promote = ["W307"]   # optional: turn the "use case with no refines" warning into a gate failure
```

Then run the gated pass with `validate --profile magicgrid`. Without the profile,
none of the `MG###` checks below fire, and the `mg_` fields are simply carried as
custom data. (The one exception is `E316` — see §3 — which is a **base-format**
check that always runs because it validates the structural `refines:` link, not an
overlay marker.)

The reports (`magicgrid`, `trade-study`, `matrix --allocations`) are **read-only**
and available **regardless of profile** — you can inspect a model's grid without
opting into the gate.

---

## 2. The grid — `mg_cell` and the `magicgrid` report

Tag any element with its grid coordinate:

```yaml
# A black-box structure element (an actor or the system block)
type: PartDef
name: ChargingStation
custom_fields:
  mg_cell: B3
```

A coordinate is a **row letter** + **column number**. The recognised set is the
twelve cells `B1`–`B4`, `W1`–`W4`, `S1`–`S4`. The pillar (column) implies a set of
legal element types:

| Col | Pillar | Legal element types |
|---|---|---|
| 1 | Requirements | `Requirement` / `RequirementDef` |
| 2 | Behaviour | `UseCaseDef` / `UseCase` / `ActionDef` / `Action` / `StateDef` / `State` |
| 3 | Structure | `Part` / `PartDef` / `Port` / `PortDef` / `Interface` / `InterfaceDef` / `Connection` / `ConnectionDef` |
| 4 | Parameters | `ConstraintDef` / `Constraint` / `CalculationDef` / `Calculation` / `AnalysisCase` |

The **`magicgrid`** report buckets every element by its `mg_cell` into the full
grid, lists and counts each cell's members, and marks **empty cells** as a
completeness hint (not an error):

```bash
syscribe -m model_mg/ magicgrid          # text grid
syscribe -m model_mg/ magicgrid --json   # cells object keyed by coordinate
```

Under `validate --profile magicgrid`, an unrecognised coordinate is `MG020` and a
type/pillar mismatch is `MG021`.

---

## 3. Refinement — `refines:` (use cases and behaviours → requirements)

A behavioural element **refines** the requirement it elaborates. Put `refines:` on
a `UseCaseDef`/`UseCase` — or on a behavioral definition (`ActionDef`/`Action`/
`StateDef`/`State`) — pointing at a `Requirement`/`RequirementDef`:

```yaml
type: UseCaseDef
name: ChargeVehicle
actors:
  - ProblemDomain::BlackBox::SystemContext::EVDriver
  - ProblemDomain::BlackBox::SystemContext::ElectricVehicle
refines:
  - ProblemDomain::BlackBox::StakeholderNeeds::FastTurnaround
custom_fields:
  mg_cell: B2
```

- **`E316`** (base-format, always runs) — a `refines:` operand that does not resolve,
  or resolves to something that is **not** a `Requirement`/`RequirementDef`.
- **`W307`** (advisory, draft-suppressed) — a non-`draft` `UseCaseDef` with no
  `refines:` link. Gate it with `--deny W307`, or promote it via the
  `[profiles.magicgrid]` profile (the `promote = ["W307"]` line above).
- The computed reverse index **`refinedBy`** lists, on each requirement, the use
  cases **and** behavioral elements that refine it — surfaced in `show` and the JSON
  `computed` block. (The `W307` "missing refines" warning stays scoped to
  `UseCaseDef`; refining `ActionDef`/`StateDef` elements still appear in `refinedBy`.)

---

## 4. Actors — `actors:` + `mg_external`, and the System of Interest (`mg_soi`)

A use case (or use-case-style requirement) lists its external **actors** with the
base `actors:` field. Each actor is a `Part`/`PartDef` marked **external** to the
system:

```yaml
type: PartDef
name: EVDriver
custom_fields:
  mg_cell: B3
  mg_external: true     # this Part is outside the system boundary
```

The single black-box system block — the **System of Interest** (cell **B3**) — is
marked instead with **`mg_soi: true`**:

```yaml
type: PartDef
name: ChargingStation
custom_fields:
  mg_cell: B3
  mg_soi: true
```

Zero `mg_soi` markers is fine; when exactly one is present the `magicgrid` report
identifies it with a **`System of interest:`** line (and a `systemOfInterest` field
in `--json`).

Under the profile:

| Code | Condition |
|---|---|
| `MG010` | An `actors:` entry resolves to no model element |
| `MG011` | An `actors:` entry resolves to an element that is not a `Part`/`PartDef` |
| `MG012` | A referenced actor `Part`/`PartDef` is not marked `mg_external: true` |
| `MG013` | A non-`draft` `UseCaseDef` has an empty or absent `actors:` list |
| `MG060` | `mg_soi: true` on an element that is not a `Part`/`PartDef` |
| `MG061` | More than one element is marked `mg_soi: true` |
| `MG062` | An element is marked **both** `mg_soi: true` and `mg_external: true` |

---

## 5. MoEs — Measures of Effectiveness (B4)

A **Measure of Effectiveness** (MoE, cell **B4**) quantifies how well a solution
meets a need. Mark a `CalculationDef` (or `AnalysisCase`) with `mg_moe: true` and
the measurement metadata:

```yaml
type: CalculationDef
name: DeliveredPowerMoE
returnType: ScalarValues::Real
expression: "moe = converterCount * converterPowerKw"
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: ProblemDomain::WhiteBox::SystemRequirements::ChargePower
  mg_moe_unit: kW
  mg_moe_direction: maximize     # maximize | minimize
  mg_moe_threshold: 150          # knock-out floor (maximize) / ceiling (minimize)
  mg_moe_objective: 360          # the target
  mg_moe_weight: 0.35            # in [0, 1]; used by trade-study
```

Under the profile:

| Code | Condition |
|---|---|
| `MG030` | `mg_moe: true` on an element that is not a `CalculationDef` or `AnalysisCase` |
| `MG031` | `mg_moe_measures` absent, or does not resolve to a `Requirement`/`RequirementDef` |
| `MG032` | `mg_moe_direction` absent or not `maximize`/`minimize` |
| `MG033` | `mg_moe_threshold`/`mg_moe_objective` not numeric or inconsistent with the direction (`maximize` ⇒ objective ≥ threshold; `minimize` ⇒ objective ≤ threshold); or `mg_moe_weight` present and not numeric in `[0, 1]` |

---

## 6. MoPs — Measures of Performance (W4/S4)

A **Measure of Performance** (MoP, cells **W4/S4**) is a concrete, internal metric
that refines a black-box MoE. Mark a `CalculationDef`/`ConstraintDef`/`AnalysisCase`
with `mg_mop: true` and point `mg_mop_refines` at the MoE it refines:

```yaml
type: CalculationDef
name: ConverterEfficiency
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: ProblemDomain::BlackBox::MeasuresOfEffectiveness::DeliveredPowerMoE
  mg_mop_unit: percent      # optional
```

The tool computes the inverse index **`mopRefinedBy`** on each MoE — the MoPs that
refine it — surfaced in `show` (and the JSON `computed` block) alongside
`refinedBy`/`actorIn`, so the MoE → MoP measurement chain is navigable from the MoE.

Under the profile:

| Code | Condition |
|---|---|
| `MG050` | `mg_mop: true` on an element that is not a `CalculationDef`/`ConstraintDef`/`AnalysisCase` |
| `MG051` | `mg_mop_refines` absent, or does not resolve (by qname/id) to a model element |
| `MG052` | `mg_mop_refines` resolves to an element that is not marked `mg_moe: true` |

---

## 7. Logical / physical layers — `mg_layer` + `matrix --allocations`

Within Structure, a `Part`/`PartDef` may declare its **layer** with
`mg_layer: logical` (W3, problem white-box) or `mg_layer: physical` (S3, solution).
A logical subsystem must be realised by a physical component through an explicit
**`Allocation`** — never a direct `supertype:`/`typedBy:` link:

```yaml
# A logical subsystem (W3)
type: PartDef
name: PowerConversionSubsystem
custom_fields:
  mg_cell: W3
  mg_layer: logical
---
# A physical component (S3)
type: PartDef
name: PowerCabinet
custom_fields:
  mg_cell: S3
  mg_layer: physical
---
# The allocation that bridges them
type: Allocation
name: AllocPowerConversion
allocatedFrom: ProblemDomain::WhiteBox::LogicalSubsystems::PowerConversionSubsystem
allocatedTo: SolutionDomain::PhysicalComponents::PowerCabinet
```

The **`matrix --allocations`** report draws the allocation source × target matrix,
rolls up unallocated sources / unused targets, and — when `mg_layer` is present —
adds a logical → physical partition:

```bash
syscribe -m model_mg/ matrix --allocations
syscribe -m model_mg/ matrix --allocations --json
```

Under the profile:

| Code | Condition |
|---|---|
| `MG040` | `mg_layer` present on a `Part`/`PartDef` but not `logical` or `physical` |
| `MG041` | A `mg_layer: logical` `Part`/`PartDef` has no `Allocation` to a `physical` element |
| `MG042` | A `logical` and a `physical` `Part`/`PartDef` share a direct `supertype:`/`typedBy:` link — relate the layers only through an `Allocation` |

---

## 8. Parametric variants — `mg_variant` Configurations

MagicGrid compares candidate solutions by **parameter set**, not by feature
selection. Mark a `Configuration` with `mg_variant: true` to make it a **parametric
variant**: the `featureModel:` requirement (`E201`) is **relaxed** — the
Configuration denotes the empty feature selection (identity projection) and is
differentiated solely by its `parameterBindings:`:

```yaml
type: Configuration
id: CONF-EVCS-001
title: "Entry tier — 2x90 kW, no spare"
status: approved
custom_fields:
  mg_variant: true
parameterBindings:
  DesignParameters.converterCount: 2
  DesignParameters.converterPowerKw: 90
```

`validate --config`, `matrix`, `diff`, and `trade-study` all treat a variant as a
normal configuration column.

Under the profile: **`MG070`** — `mg_variant: true` on an element that is not a
`Configuration`.

> The bundled `model_mg/` configurations use a real feature model; `mg_variant`
> is the alternative when you want pure parameter variants with no feature tree.

---

## 9. The trade study — `trade-study`

MoEs only earn their keep when they **drive solution selection**. The
**`trade-study`** report scores every `Configuration` against the model's MoEs:

```bash
syscribe -m model_mg/ trade-study
syscribe -m model_mg/ trade-study --config CONF-EVCS-001 --config CONF-EVCS-002
syscribe -m model_mg/ trade-study --json
```

- **Rows** — every `mg_moe` element; **columns** — the model's `Configuration`s (or
  the subset named by repeated `--config`).
- For each (MoE, Configuration), the MoE's `expression:` is evaluated, resolving
  each variable from the configuration's `parameterBindings:`. The value is
  normalised to a score in `[0,1]` against `mg_moe_threshold`/`mg_moe_objective` per
  the `mg_moe_direction`. A value worse than the threshold scores `0` and is a
  **knock-out** (`!`).
- A footer reports each configuration's weighted total (`mg_moe_weight × score`).
  The top non-failing configuration is marked **`WINNER`**; any with a threshold
  violation is **`FAIL`**.

```text
Rollup (weighted total per configuration):
  Entry tier — 2x90 kW, no spare       0.225
  Premium tier — 4x90 kW, dual spare   0.814 WINNER
```

---

## 10. Finding-code reference

All `MG###` findings are **Error** severity and fire **only** under
`validate --profile magicgrid`. `E316` and `W307` concern the `refines:` link;
`E316` runs in the base format, `W307` is advisory until promoted.

| Code | Severity | Gating |
|---|---|---|
| `E316` | Error | base format — always runs |
| `W307` | Warning | advisory; `--deny W307` or promote via the magicgrid profile |
| `MG010`–`MG013` | Error | actors (§4) — magicgrid profile |
| `MG020`–`MG021` | Error | `mg_cell` grid (§2) — magicgrid profile |
| `MG030`–`MG033` | Error | MoEs (§5) — magicgrid profile |
| `MG040`–`MG042` | Error | `mg_layer` logical/physical (§7) — magicgrid profile |
| `MG050`–`MG052` | Error | MoPs (§6) — magicgrid profile |
| `MG060`–`MG062` | Error | System of Interest (§4) — magicgrid profile |
| `MG070` | Error | `mg_variant` (§8) — magicgrid profile |

See the [Rule Reference](../validation/rules.md#magicgrid-overlay-e316-w307-mg010mg070)
for the per-code conditions.

> **Root-name cross-reference hint (`REQ-TRS-XREF-006`).** When an unresolved
> reference (including a `refines:` operand flagged by `E316`) begins with the root
> package's `name:` followed by `::`, and the *stripped* remainder resolves, the
> tool appends an advisory hint naming the corrected reference. Qualified names are
> relative to the model root, so the root package contributes **no** segment — write
> `ProblemDomain::…`, never `<RootName>::ProblemDomain::…`. The hint is advisory: it
> never changes resolution and never rewrites the model.

---

## Further reading

- [Custom Fields](custom-fields.md) — the `custom_fields:` map that carries every `mg_` marker
- [Traceability](traceability.md) — `Allocation` and the §12 rules
- [Variability & Product Lines](variability.md) — `Configuration`, `parameterBindings`, `matrix`, `diff`
- [Rule Reference](../validation/rules.md) — the full E/W/MG finding catalogue
- the worked **`model_mg/`** example model in the repository
