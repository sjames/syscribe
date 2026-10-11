# Fault injection tied to FMEA/FTA — design

GH #306 (part of #272), requirement `REQ-TRS-FAULT-001`.

## Problem

The safety analyses and the executable models are separate worlds: an FMEA row is prose ("command lost", "sensor stuck"), a fault-tree event carries a failure rate. Nothing shows what the *system* does when that failure happens. The composite engine can: it has parts, ports, messages and variables. What is missing is a machine-readable fault model attached to the analysis elements, and a way for scenarios to switch it on.

## Fault model

`injection:` is a list on an `FMEAEntry` row and on a `FaultTreeEvent`; the element's id is the fault's id and its `ref:` names the part definition that fails. Item kinds — each is a deviation the engine can realise deterministically:

| kind | what it models | engine |
|---|---|---|
| `stuck` | a stuck-at sensor or register | `Run::pin(Pin::Stuck)`: every write to the variable is replaced |
| `offset` | a biased measurement | `Run::pin(Pin::Offset)`: every write is increased by the offset |
| `silent` | fail-stop of a part | `Run::silence`: no events, no timers, no messages; variables frozen |
| `drop` / `delay` / `corrupt` | loss, lateness or corruption on a connection | applied in `System::collect` as the message leaves the part |

The kinds are the classic set of fault-injection (stuck-at, bias, fail-silent, omission, timing, value) and deliberately small: each one has an unambiguous deterministic semantics, which a probabilistic or time-triggered fault would not (time-triggered activation is a scenario step: `When 2 s elapses` then `When fault … is injected`).

## Resolution

`SystemModel::build` takes every FMEA row and fault-tree event with a well-formed `injection:` and resolves its items against the composite: an item applies to the part named by `part:`, or to every part whose definition is, or specialises, the element's `ref`. A fault with no applicable part is left out; one that applies but does not fit (unknown variable, stuck value of the wrong kind or dimension, a port that is not an out port) is recorded with its problems and reported as `E936` when a scenario **injects** it, so one unfitting analysis row cannot break every scenario of the composite.

**Semantics worth knowing.** A value fault applies to every *write* of the variable; `offset` therefore compounds on a write that starts from the current value (`x := x + 1` grows by 2 under an offset of 1), which is how a biased register behaves. Several faults on one variable stack, each pin is tagged with its fault, and clearing one leaves the others. A bound variable that is pinned is compared with what it would store, so bindings do not add trace steps. `corrupt` of a field the payload does not have adds the field. Faults are part of the evidence hash of the composite.

## Run time

`System::inject(id)` activates the items, records "fault X injected at t s", and runs the system to quiescence; `clear_fault(id)` removes pins and port faults (a silent part stays failed). Scenario steps: `Given|When fault <id> is injected`, `When fault <id> is cleared`. The verdict lists the fault events first, then the messages, and a message lost to a fault says so. An injected fault is part of the scenario's text, so it is part of the evidence a recorded run keeps.

## The campaign (phase 2, REQ-TRS-FAULT-002)

`syscribe inject <composite|TestCase>` answers "does the design survive this failure?" without a second set of scenarios. The notion of *safe* is the TestCase's own: a scenario tagged `@invariant` (Gherkin tags are recorded by the extractor; a tag before `Feature:` applies to all) states a property that must hold whatever fails. The campaign

1. checks the invariants pass **without** faults (a property that is false without faults cannot judge them);
2. builds the fault sets: every fault that applies to the composite on its own, restricted by `--fmea <sheet>`, plus — with `--ft <FaultTree>` — every minimal cut set up to `--max-order` (default 2) whose events all have a fault model (a cut set with an event that has none is listed as *not run*; common-cause pseudo-events fall under that);
3. injects each set at time 0 and runs every invariant unchanged: **survived** (all pass), **defeated** (one fails; the invariant and the failing step are reported), **error** (the run could not be made);
4. reports which FMEA rows / fault-tree events of the composite's parts were exercised, which have a fault model but were not, and which are prose only.

A set in which an invariant fails is **defeated** even when another invariant of the same set could not be evaluated. A campaign with no set to run is a usage error (exit 1), never a vacuous pass. Common-cause pseudo-events (`CCF:<group>`) stand for every member failing together. Faults go in before the first step, so a `Given` that sets a stuck variable is discarded; invariants must not contain fault steps themselves. Exit 0 when every set survived, 2 on any defeat or error (`--report-only` reports and exits 0), 1 for usage errors. The fixed order (single faults in element order, then cut sets by order) makes the report reproducible.

A defeat is the useful result: it names a failure mode (or a combination) under which a stated safety property is violated, and the trace of that run (`simulate` with the fault steps) shows how. The cut-set mode connects the quantitative fault tree to the executable model: the sets the tree says are minimal are the ones tried.

**Later:** faults injected after time 0, sampled sets, and quantitative use of the probabilities.
