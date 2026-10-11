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

## Phase 2 (not here)

A campaign: run the scenarios under every single FMEA fault and every minimal cut set of a fault tree up to an order, and report which fault sets the design survives, and which FMEA rows are exercised by an injection run at all.
