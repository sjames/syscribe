# System composition

Several parts, each with its own behaviour, can be executed together: every part runs its state machine, messages travel along the port connections the model already declares, values travel along binding connections, and all of it runs on one deterministic virtual clock (GH #297; requirement `REQ-TRS-SYSCOMP-001`, design note `docs/design/system-composition.md`). The checks `E930`–`E934` and `W936` apply to a composite that has at least one part with an executable state machine; every other model sees nothing new.

## A small system

```yaml
---
type: StateDef
name: Sensor
variables:
  - {name: level, value: "0"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Idle, accept: Sample, effect: {send: Reading, via: out, payload: {value: "level"}}}
---
```

`send` is an effect (also allowed in `initialEffect`, `entryAction` and `exitAction`): the event, the port of the owning part it leaves through, and a payload of expressions evaluated at send time. Within one effect `assign:` runs before `send:`.

<!-- syscribe-example: skip reason="a fragment of a composite shown beside the machines it refers to" -->
```yaml
---
type: PartDef
name: System
features:
  - {name: sensor, typedBy: SensorPart}
  - {name: ctl, typedBy: ControllerPart}
connections:
  - {from: sensor.out, to: ctl.inp, delay: "2 ms"}
bindingConnections:
  - {from: sensor.level, to: ctl.level}
---
```

* Ports are the `features:` of `type: Port` of the part definition and its **child `Port` elements** (a file `Def/out.md` with `type: Port` and `direction:` is `Def::out`); ports of a supertype are inherited.
* A composite **inherits** the `features:` (parts), `connections:`, `bindingConnections:` and `plantStep:` of its supertypes: a part of the same name is overridden by the nearest definition, the base's connections and bindings come first, and a subtype's connection with the same two ends (or binding into the same variable) replaces the base's in place, so a variant can change a `delay:` or re-source a variable. Only `supertype:` is followed (a part typed by a composite does not inherit its wiring), and a finding that comes from the inherited wiring is reported on the base, not again on every subtype. A variant therefore only states what it adds or replaces.
* A part's behaviour is the one `StateDef` named by `exhibitsStates:` on its definition (or an ancestor). A part with none is inert.
* `connections:` join an out/inout port to an in/inout port. The message arrives as the event `<port>/<event>`, the same name `accept: {payload: Reading, via: inp}` listens for. `delay:` (default none) postpones delivery.
* `bindingConnections:` keep a variable of one part equal to a variable of another, in the unit of the sink. Types and unit dimensions must agree (`E933`).

## Semantics

* **One clock.** The next instant is the earliest of a part's timer and a pending delayed message. Everything due at one instant is processed until nothing changes before time moves on.
* **Messages** are asynchronous and reliable. Each is one run-to-completion step of the receiver. Delivery order is fixed: by due time, then by emission order; one output connected to several inputs delivers in connection order. A message sent through a port that nothing is connected to is recorded and goes nowhere.
* **Bindings** are copied after every settle, source to sink, until nothing changes (at most 1000 rounds).
* **Livelock.** More than 100,000 messages at one instant is an error that names the parts answering each other.
* **No threads, no wall clock:** the same model and scenario always produce the same trace.

## Scenarios

A `TestCase` whose `subject:` is the composite uses the Gherkin step vocabulary with part names; see `syscribe help simulate`.

```gherkin
Scenario: the command arrives after the link delay
  When sensor receives Sample
  Then sensor sent Reading via out
  When 3 ms elapses
  Then ctl.last == 7
```

`Then <part> sent …` looks at every message the part sent earlier in the scenario. `simulate --json` and `--record` carry a `messages` list in each verdict (who sent what, when, and when it was delivered); the trace steps are tagged with their part, and the step a delivered message caused in the receiver carries a note naming the sender, the port and the send time, so the timeline shows the message beside the steps it links. The machine hash of a composite covers every part's machine hash and the wiring.

## Coverage

`simulate <TestCase|composite> --coverage` reports every part that has a state machine with the usual state, transition, pair, guard-condition (MC/DC) and timer coverage, labelled `<composite>.<part> (<machine>)`, merged over the scenarios that ran, followed by an aggregate of states, transitions and guard conditions. `--min-states`, `--min-transitions` and `--min-guards` are checked for every part, and the part below a threshold is named (exit 2). In `--json`, `coverage` is `{composite, hash, parts: […], aggregate}`.

## Mutation

`syscribe mutate <composite|TestCase>` seeds one fault at a time into the machine of each part (the operators of `syscribe help mutate`) and judges every mutant by running all the composite's scenarios with that one part replaced, so a weak scenario set shows up as a surviving mutant under the part it concerns. `--part <name>` limits the analysis to one part; `--min-score` is checked for every part. Composites with FMU parts work too: each mutant run gets fresh plants.

## Fault injection

The failure modes of the safety analyses are injectable (`injection:` on an `FMEAEntry` row or a `FaultTreeEvent`, see [Safety & Security Analysis](safety-analysis.md#injectable-failure-modes--injection)). A scenario injects and clears them:

```gherkin
Scenario: the receiver stays safe when the command is lost
  Given fault FM-LINK-001 is injected
  When tx receives Tick
  Then tx sent Cmd via out
  And rx is in state Waiting
  When fault FM-LINK-001 is cleared
```

The verdict lists every injection and clearing with its time among its messages, and a message lost to a fault says why. Faults are deterministic; a part that failed silent stays failed. A fault that applies to the composite but does not fit it (an unknown variable, a stuck value of the wrong dimension) is `E936` when a scenario injects it; the other scenarios are unaffected. An unknown id in a step is `E937`. Faults are part of the composite's evidence hash.

## Diagnostics

| Code | Meaning |
|---|---|
| `E930` | a part has more than one behaviour, or its `exhibitsStates` does not resolve to a StateDef |
| `E931` | a connection end is not a port of that part, or the direction is wrong |
| `E932` | a machine sends or accepts through a port its part does not have |
| `E933` | a binding names an unknown variable or joins different types or dimensions |
| `E934` | malformed composite: unnamed or duplicate part, `delay:` not a duration |
| `E936` | a fault model (`injection:`) is malformed, or does not fit the composite it applies to |
| `E937` | a `fault … is injected/cleared` step names an unknown fault |
| `W936` | an event is sent through a connection but never accepted at the other end, or the reverse |

## FMU plants as parts

With the `fmi` feature a part whose definition carries `fmu:` is a plant (see [FMU plants](fmi-plants.md)). The composite declares the communication step, and bindings join plant variables to machine variables:

```
plantStep: "10 ms"
bindingConnections:
  - {from: ball.h, to: ctl.height}      # plant output → machine variable
  - {from: ctl.thrust, to: ball.thrust} # machine variable → plant input
```

The loop, units and evidence are those of the FMI page: inputs are held over a step, the step is cut at the earliest timer or delayed message, outputs are read at the boundary and the machines then settle; every scenario starts with fresh plants and the verdict lists their provenance. Direction, type and unit dimension of each plant binding are checked before the run. Outputs of one communication point reach the machines together, before any timer due at that instant fires. A variable has at most one source (`E933`), and a binding needs parts that have behaviour. `validate` checks the plant files and `plantStep`; the direction, type and unit dimension of each plant binding are checked when `simulate` builds the system. The evidence hash covers the communication step and every plant's file hash and parameters. A composite with a plant part is refused (exit 1) in a build without the feature.

## Not in this version

Mutation of the wiring (connections, bindings, delays) and of plant parameters, coverage of plant parts and of messages, action flows as part behaviour, parts with a multiplicity above one, lossy or reordering channels.
