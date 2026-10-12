# In-process plants

A composite closes its loop around physics with a **plant part**. Besides an FMU (see [FMU plants](fmi-plants.md), behind the `fmi` build feature), a plant can be written in the model itself: a `PlantDef` in the expression language, or one of a few standard plants. Nothing outside Syscribe is needed, there is no feature flag, and the same model and scenario give the same trace, bit for bit, on every machine (GH #340): the engine uses IEEE arithmetic, integer powers by repeated squaring and the software `libm` functions, never a platform's own math library, and a test pins the bits of a reference response.

## A first-order lag

```yaml
---
type: PlantDef
name: Lag
parameters:
  - {name: k, value: "1"}
  - {name: tau, value: "200 ms"}
inputs:
  - {name: u}
outputs:
  - {name: y}
states:
  - {name: x, value: "0"}
equations:
  - "der(x) = (k * u - x) / tau"
  - "y := x"
---
```

A part becomes the plant with `plant: Lag`, and the composite declares the communication step and the bindings, exactly as for an FMU:

```yaml
---
type: PartDef
name: Sys
plantStep: "10 ms"
features:
  - {name: ctl, typedBy: CtlP}
  - {name: lag, typedBy: LagP}     # LagP carries `plant: Lag`
bindingConnections:
  - {from: ctl.u, to: lag.u}       # machine variable → plant input
  - {from: lag.y, to: ctl.y}       # plant output → machine variable
---
```

## The declaration

| Field | Meaning |
|---|---|
| `parameters:` | `{name, value}`; `value` is a constant with its unit (`"0.4 SI::ohm"`) |
| `inputs:` | `{name, unit, value}`; written by bindings, `value` until the first write. A `unit` is a coherent SI unit (`V`, `A`, `ohm`, `s`): the plant computes in base units, so a scaled unit such as `mV` is refused (`E938`) rather than read as `V` |
| `outputs:` | `{name, unit}`; read by bindings |
| `states:` | `{name, value}`; the initial value fixes the dimension |
| `equations:` | `der(state) = expression` for each state, `output := expression` for each output |
| `method:` | `rk4` (default) or `euler`, fixed step |
| `maxStep:` | the longest integration sub-step; each communication step is split evenly below it, at most one million sub-steps per communication step |

Every equation is checked for unit dimension before anything runs: `der(x)` must have the dimension of `x` per second, an output the dimension it declares. Outputs may use earlier outputs; a cycle between them is an algebraic loop. Besides the operators of the expression language, `exp`, `ln`, `sin`, `cos` and `tanh` take a dimensionless argument.

A part may override parameters, and the standard plants need no definition:

```yaml
plant: {def: Lag, parameters: {tau: "50 ms"}}
plant: {library: FirstOrderLag, unit: "SI::V", parameters: {tau: "5 ms"}}
```

| Library plant | Behaviour | Parameters |
|---|---|---|
| `Gain` | `y := k·u` | `k` |
| `FirstOrderLag` | `der(x) = (k·u − x)/tau`, `y := x` | `k`, `tau` |
| `Integrator` | `der(x) = k·u`, `y := x` (`unit` is the output's) | `k` |
| `Saturation` | `y := min(max(u, lo), hi)` | `lo`, `hi` |
| `DelayLine` | `y` is `u` sampled at the communication points and delayed by `delay`; `initial` before. The line keeps only the samples the current delay needs, so a delay raised mid-run by a fault shows `initial` until it has caught up | `delay` (not negative), `initial` |
| `ImpedanceDivider` | `v := vSrc − z·i` (input `i` in A, output `v` in V) | `vSrc`, `z` |

## Semantics

The loop is the one FMU plants use: machine variables bound to plant inputs are written and held over the coming step, the step is cut at the earliest timer or delayed message, the plant integrates to the boundary, and the outputs reach the machines together before any timer due at that instant. Every scenario starts with fresh plants. When every derivative is exactly zero the states are at rest and the rest of the step is skipped, which changes no bit of the result and keeps long, quiet scenarios cheap. The evidence hash of the composite covers each plant's parameters, equations, method and `maxStep`, and a recorded run lists each plant as provenance.

## Faults

An FMEA row or fault-tree event can act on a plant part (see [Safety analysis](safety-analysis.md)): `stuck` or `offset` on an output, `silent` (the plant freezes and the machines keep its last outputs), and `{kind: parameter, parameter: k, value: "2"}`, which changes a parameter until the fault is cleared.

## Worked example

The grid-tie inverter (`model_gti`) runs its anti-islanding perturbation check against `Behavior::Plants::GridImpedance`, the line voltage's first-order answer to the current the inverter injects. A connected grid answers with a small change, an islanded or weak grid with a large one, and sensing chains that read a fixed value with none. That is what turns the sensing common cause FTE-GTI-009 from a single cut set into a cut set of order 2 (see the [grid-tie inverter page](../demo-model/grid-tie-inverter.md)).

## Diagnostics

| Code | Meaning |
|---|---|
| `E930` | a part carries both `fmu:` and `plant:` |
| `E933` | a binding reads something that is not a plant output, writes something that is not a plant input, joins different dimensions, or joins two plants (a plant exchanges values with machines; two plants that act on each other are one `PlantDef`) |
| `E934` | a composite with an in-process plant part has no `plantStep:` |
| `E935` | a unit or type error in an equation |
| `E936` | a fault does not fit the plant (an unknown output or parameter, a value of the wrong dimension) |
| `E938` | a malformed `PlantDef`: structure, equation forms, unknown names, an algebraic loop, `method`, `maxStep`; or a plant key on another element type |
| `E939` | a `plant:` declaration that does not resolve, names an unknown library plant, or overrides a parameter that does not exist or has the wrong dimension |

## Design note

The plant is a `Plant` implementation like the FMU wrapper, so the composite engine, fault injection, recording and coverage did not change shape. The ODEs are lowered once to the expression IR; the integrator is fixed-step RK4 (or Euler) with inputs held over each step, the only scheme whose result does not depend on the platform's floating-point libraries beyond IEEE arithmetic and `libm`. Variable-step solvers, events inside a plant (zero crossings), algebraic equations solved for an unknown, and arrays are out of scope: a stiff or event-driven plant still belongs in an FMU.
