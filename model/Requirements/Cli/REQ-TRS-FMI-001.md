---
type: Requirement
id: REQ-TRS-FMI-001
name: "A state-machine run can exchange values with FMI Co-Simulation FMU plants on the shared virtual clock, behind the fmi feature"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - fmi
---

Scenarios of a state machine shall be able to run against plant models supplied as FMI 2.0 or 3.0 Co-Simulation FMUs, with syscribe as the master (GH #291, part of #272). The whole feature is gated by the Cargo feature `fmi`, off by default. Rationale and the co-simulation algorithm: `docs/design/fmi-plant-cosimulation.md`.

## Behavior

- **Gating.** Everything that needs an FMI library is compiled only with `--features fmi` on the `syscribe` crate. Without it the default build has no FMI dependency and no behaviour change: the fields below still parse (no `W047`), no FMI check runs, one `I011` finding per element that declares `fmu:`, `plants:` or `plantConnections:` says that FMI support is not compiled in, and `simulate` of a TestCase that uses a plant exits `1` with that message. The seam is a dependency-free `Plant` trait in `syscribe-model`; the `syscribe-fmi` crate implements it with the `fmi` crates and is an optional dependency of `syscribe`.
- **Plant declaration.** A `Part`/`PartDef` may carry `fmu: {file, fmiVersion, kind, sha256, parameters}`. `file` is resolved like `implementedBy:` (local path or remote URI); `kind` is `cosimulation` (the only kind in v1); `sha256` pins the artifact; `parameters` are values for the FMU's parameters, in the FMU's units. With the feature on: `E888` for a malformed declaration (unknown key, `kind` other than `cosimulation`, `parameters` naming a variable that is not a parameter of the FMU or whose value does not evaluate); `E966` when the file cannot be read, is not an FMU, its `fmiVersion` differs from the one in `modelDescription.xml`, it does not support Co-Simulation, or its hash differs from `sha256`.
- **Wiring.** A TestCase with a state-machine `subject:` may carry `plants: [<Part>…]`, `plantStep: <duration>` (the maximum communication step) and `plantConnections: [{from, to}…]`. A connection runs from a plant output (`<Plant>.<variable>`) to a machine variable, or from a machine variable to a plant input or tunable parameter. With the feature on, `E889` for a connection that names an unknown plant or variable, goes the wrong way for the variable's causality, has a unit dimension different from the FMU variable's, or uses an unsupported type (supported: Real/Float64/Float32, Integer, Boolean).
- **Co-simulation loop.** `advance(Δ)` of a run with plants is the loop of the design note: the discrete side settles, mapped machine variables are written to plant inputs (held constant over the step), every plant takes a `do_step` to the next time, which is the earlier of the next multiple of `plantStep`, the earliest discrete due time (a timer or timed accept) and the target; plant outputs are read into the mapped machine variables; the discrete side settles again, so a `when:` over a plant output is evaluated at every step boundary. The plant is not interpolated between boundaries.
- **Initialisation.** Parameters and start values are set, each FMU enters and exits initialization mode, and the initial outputs are read before the machine's first settle. Every scenario or script run instantiates fresh plants.
- **Units.** Values cross the boundary in SI base units, converted with the FMU's declared unit factor and offset; the dimension of every connection is checked once, before the run starts.
- **Scenarios.** The step vocabulary of REQ-TRS-SMSCEN-001 is unchanged; plant outputs are ordinary machine variables through their connections. `When <duration> elapses` runs the loop; `When <variable> becomes …` on a mapped input reaches the plant at the next step boundary.
- **Errors.** An FMU error or discard status, a request to terminate, or a value that is not finite stops the run; the message and the plant name are recorded as the final `error` trace step.
- **Evidence and determinism.** A recorded run carries the provenance of every plant: FMU file hash, FMI version and kind, GUID/instantiation token, communication step, parameters, target triple and the version of the FMI wrapper. The trace records the values exchanged with each plant at every settle point (changes only). Results are reproducible for the same FMU binary on the same platform; across platforms assertions on plant values should use tolerances.
- **Not in v1:** Model Exchange, SSP packages, FMI clocks and Scheduled Execution, variable-step master algorithms with rollback, plants with different step sizes, action flows with plants, exporting syscribe behaviour as an FMU.
