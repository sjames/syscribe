# FMU plants (FMI co-simulation)

A state machine usually controls something physical. With the optional `fmi` build feature, syscribe can run an **FMU** (Functional Mock-up Unit, FMI 2.0 or 3.0 Co-Simulation) as the plant next to a `StateDef`, on the same virtual clock as the machine, so Gherkin scenarios can check a controller against a real plant model (GH #291; requirement `REQ-TRS-FMI-001`, design note `docs/design/fmi-plant-cosimulation.md`).

Syscribe is the **master**: it steps the FMU, feeds it machine variables, reads its outputs back into machine variables and settles the machine. FMI is a Modelica Association standard, not an ISO one. SSP, Model Exchange, scheduled-execution FMUs, clocks and remote FMUs are not supported in v1.

## Build gating

The feature is off by default: `cargo build --features fmi` (it needs a C toolchain with libclang for the FMU loader).

| Build | Behaviour |
|---|---|
| default | `fmu:`, `plants:`, `plantStep:`, `plantConnections:` parse; nothing is checked; `validate` reports `I011`; `simulate` of a TestCase that declares plants exits `1` with the reason |
| `--features fmi` | declarations, files and wiring are checked (`E888`, `E889`, `E966`) and scenarios co-simulate |

## Declaring the plant

A `Part` carries the FMU:

<!-- syscribe-example: expect I011 reason="the default build has no FMI backend, so plant fields are reported as unchecked" -->
```yaml
---
type: Part
name: Ball
fmu:
  file: BouncingBall-2.0.fmu
  fmiVersion: "2.0"
  kind: cosimulation
  sha256: 1604856668b4c6081109e2ef0b778ef725a285095373bd7a54b24c7a9d6faad4
  parameters: {e: 0.7}
---
```

`file` is local (relative to the model root); `sha256` pins the exact binary; `parameters` are set before the run, in the FMU's units. A TestCase selects plants and wires them to the machine under test:

<!-- syscribe-example: expect I011 reason="the default build has no FMI backend, so plant fields are reported as unchecked" -->
```yaml
---
type: TestCase
id: TC-FMI-001
name: ball
status: active
testLevel: L2
subject: Ctl
verifies: [REQ-FMI-001]
plants: [Ball]
plantStep: "10 ms"
plantConnections:
  - {from: Ball.h, to: height}
---
```

A connection is `Plant.variable` → machine variable (a plant output) or machine variable → `Plant.variable` (a plant input). Parameters are not wired; set them under `fmu: parameters:`. Plant names must be distinct. Direction, type and unit dimension are checked before anything runs.

## Semantics

* Communication points lie on a grid at multiples of `plantStep`; a step is cut short at the earliest discrete timer of the machine, so timers still fire at their exact due time.
* Inputs are held constant over a step (zero-order hold). Outputs are read at the end of the step and the machine then settles, so a `when:` trigger fires at the **first step boundary after the crossing**; there is no interpolation. Initial outputs are read before the first settle.
* Values cross the boundary in **SI base units** (the FMU's `factor` and `offset` are applied); `Real`/`Float32/64`, `Integer` and `Boolean` are supported.
* Every scenario instantiates fresh plants, so scenarios cannot affect each other.

## Limits of v1

* The initial outputs (time 0) are read right after initialization, before the machine's variables are written to the plant's inputs; inputs take effect from the first step.
* `Float32` and array variables, strings and binaries are not supported (`E889`).
* More than 5,000,000 communication steps in one `When … elapses` is refused; use a larger `plantStep`.
* Only the `syscribe` CLI built with the feature registers the backend; the web server reports `I011`.

## Evidence

`simulate --json` and the trace files written by `--record` carry, per plant, the FMU `sha256`, FMI version, model name, GUID, step, backend and platform. `simulate` refuses to start (`E966`) if the file no longer matches the declared `sha256`. Coverage works with plants; `mutate` refuses TestCases that use plants.

## Diagnostics

| Code | Meaning |
|---|---|
| `E888` | malformed `fmu:`, `plants:`, `plantStep:` or `plantConnections:` |
| `E889` | a connection with an unknown variable, wrong direction, wrong type or mismatched unit dimension |
| `E966` | the FMU is missing, remote, not an FMU, has no Co-Simulation interface, has another `fmiVersion`, or does not match `sha256` |
| `I011` | plants are declared but this build has no FMI support |
