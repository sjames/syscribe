# FMI plant co-simulation — design

`GUIDE · DESIGN` · Epic GH #291 · Requirement: `REQ-TRS-FMI-001` · Status: proposed · Part of the execution epic #272

Syscribe's interpreters run discrete behaviour (state machines, action flows). Physical parts of a system — a battery, a motor, a thermal loop, vehicle dynamics — are continuous and are modelled in other tools. **FMI** (Functional Mock-up Interface, a Modelica Association standard, versions 2.0 and 3.0 supported here) is the open interface those tools export: an **FMU** is a zip with a `modelDescription.xml` and a compiled model. This feature lets a state-machine run exchange values with FMU *plants* on a shared virtual clock, so a scenario can say "the throttle ramps for 2 s and the overshoot stays within limit" against a real plant model instead of a faked variable.

It is **import only**: syscribe orchestrates, the FMU is the plant. Exporting a syscribe model as an FMU is a different feature and not part of this one.

## Gating

Everything that needs an FMI library is behind the Cargo feature **`fmi`**, **off by default**, on the `syscribe` binary crate (`cargo build --features fmi`). The default build compiles none of the FMI dependencies (the `fmi` crates pull about 190 packages and `fmi-sys` runs `bindgen`, which needs `libclang` at build time — to be confirmed in CI), so existing builds, CI and the qualification suite are unchanged.

| | feature off (default) | feature on |
|---|---|---|
| `plant:` / `fmu:` fields | parsed, no `W047` | parsed |
| checks (`E888`, `E889`, `E966`) | not run; one `I011` finding per element that declares them says FMI support is not compiled in | run |
| `simulate` of a TestCase that uses a plant | exit 1: "this build has no FMI support (cargo build --features fmi)" | runs |
| everything else | unchanged | unchanged |

The seam is a small trait in `syscribe-model` that is always compiled and has no dependencies (below). Only the implementing crate, `syscribe-fmi`, depends on `fmi`, and only the `syscribe` binary links it, under the feature.

```
syscribe-model   trait Plant { … }          ← interpreters call it; no FMI knowledge
syscribe-fmi     impl Plant for FmuPlant    ← optional dependency, uses `fmi`, `fmi-schema`
syscribe (bin)   [features] fmi = ["dep:syscribe-fmi"]
```

## Model

A plant is an ordinary model element with an FMU artifact, so it is traceable (`satisfies`, allocation) like any `Part`:

```yaml
type: Part
name: BouncingBall
domain: system
fmu:
  file: Plants/BouncingBall.fmu     # resolved like implementedBy: (local or remote)
  fmiVersion: "3.0"                 # checked against modelDescription.xml
  kind: cosimulation                # only Co-Simulation in v1
  sha256: 9f2c…                     # pins the artifact; E890 on mismatch
  parameters: {e: 0.8}              # tunable parameters, in the FMU's units
```

A `TestCase` (or later a run configuration) wires plants to the subject:

```yaml
type: TestCase
subject: Controller                  # the StateDef under test
plants: [Plants::BouncingBall]
plantStep: "10 ms"                   # maximum communication step
plantConnections:
  - {from: BouncingBall.h, to: height}   # plant output → machine variable
  - {from: thrust, to: BouncingBall.u}   # machine variable → plant input
```

`BouncingBall.h` is the FMU variable `h`, found by name in the model description. Direction follows from the `from`/`to` ends: an FMU output or calculated parameter can only be a `from`; an FMU input or tunable parameter only a `to`.

## Co-simulation semantics

Syscribe is the **master**; it keeps the virtual clock (REQ-TRS-SMEXEC-001). `advance(Δ)` runs this loop with the plants:

```
t = now; target = now + Δ
while t < target:
    t_next = min(target, t + h, earliest discrete due time (timers, timed accepts) in (t, target])
    1. settle the discrete side at t            (state machines, actions to quiescence)
    2. write  mapped machine variables → plant inputs      (held constant over the step)
    3. do_step(t, t_next − t) on every plant
    4. read   plant outputs → mapped machine variables      (type and unit checked)
    t = t_next
    5. settle the discrete side at t            (change triggers `when:` see the new outputs here)
```

Consequences, all deliberate and documented:

* **Zero-order hold.** An input is constant over a step; a discrete change takes effect at the next step boundary.
* **Event resolution is `h`.** A `when:` over a plant output fires at the first step boundary at or after the crossing. The plant is not interpolated. Timers still fire at their exact due time, because the step is cut there (FMI allows a variable communication step).
* **Initialisation.** Parameters and start values are set, the FMU enters and exits initialization mode, and the initial outputs are read **before** the machine's first `settle`, so an initial `when:` sees them.
* **One plant instance per run.** Every scenario instantiates fresh plants, so scenarios are independent. Mutation runs multiply FMU runs; that is a performance cost, not a semantic one.
* **State machines only in v1.** Action flows have no continuous time; they can use the same hook later.
* **Co-Simulation only in v1.** Model Exchange needs an integrator in the master; the plant trait leaves room for it.
* An FMU that returns an error or discard status, or asks to terminate, stops the run with the message recorded as an `error` trace step.

## Units and types

The FMU declares units as SI base-exponent maps (`UnitDefinitions`, e.g. `m/s` is `m=1, s=-1`) with a `factor` and `offset` to SI. Syscribe's `Dim` is the same representation. At load time each connection is checked: the dimension of the machine variable must equal the FMU variable's unit dimension (`E889`), otherwise the connection is rejected before anything runs. Values cross the boundary in **SI base units** (`value × factor + offset`, and back). Supported variable types in v1: Real/Float64/Float32, Integer, Boolean. Strings and binaries are rejected (`E889`).

## Determinism and evidence

An FMU is native code; its results are reproducible for the same binary on the same platform (the spike reproduced a BouncingBall trace to the last digit across two independent Rust wrappers and across FMI 2.0 and 3.0), but not guaranteed across platforms or solver versions. So:

* The recorded evidence carries the **provenance of the plant**: FMU file hash, FMI version and kind, `modelDescription` GUID/instantiation token, the communication step, the parameters, the target triple and the `fmi` crate version. A changed FMU is detected like a changed machine (the IR hash already identifies the machine).
* Scenarios should assert plant-dependent values with a tolerance (`Then abs(height - 0.5) < 0.01`); the expression language already supports it.
* The trace records, at every settle point, the values read from and written to each plant (compact, only changes).

## Tests

Fixtures are three small, BSD-licensed **Reference FMUs** vendored with their licence and checksums under `crates/syscribe-fmi/tests/fmus/` (BouncingBall 2.0 and 3.0, Feedthrough 3.0), so tests never download anything. The spike established the expected numbers: BouncingBall from 1 m with step 0.01 s first goes below 0.1 m at t = 0.43 s, h = 0.09517465, v = −4.2183. Gated tests run in a CI job that builds with `--features fmi`.

## What is deliberately not here

SSP package reading (syscribe's own connections are the system structure), Model Exchange, FMI clocks and Scheduled Execution, variable-step master algorithms with rollback, state save/restore, multiple plants at different step sizes (all plants share `h`), exporting syscribe behaviour as an FMU.

## Implementation notes

* `fmi-schema` 0.8 drops the variable-level `unit` of FMI 2 `Real`, so the backend reads `modelDescription.xml` itself (quick-xml) for units and variable metadata.
* The declared `sha256` is enforced both by `validate` (`E966`) and by `simulate` before any scenario starts.
* Plant provenance is nested per plant as `{name, step, plant: {fmiVersion, sha256, backend, platform, ...}}`.
