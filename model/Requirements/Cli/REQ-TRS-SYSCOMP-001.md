---
type: Requirement
id: REQ-TRS-SYSCOMP-001
name: "A composite part can be executed: the behaviours of its parts run on one virtual clock, communicating over the connections of the model"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - composition
---

A composite `Part`/`PartDef` whose parts carry behaviour shall be executable as a system (GH #297, part of #272): every part runs its own state machine, messages travel along the port connections the model already declares, values travel along binding connections, and all of it runs on one deterministic virtual clock. Rationale and algorithm: `docs/design/system-composition.md`.

## Behavior

- **Machine output.** A transition, entry, exit or initial effect may `send` an event with an optional payload through one of the owning part's ports: `effect: {send: Cmd, via: out, payload: {level: "x + 1"}}`. A machine without `send` has an unchanged IR and hash. A `send` outside a composition is recorded in the trace and otherwise has no effect.
- **Composite.** A composite is a `Part`/`PartDef` with `features:` that are parts (`typedBy:` a `PartDef`). A part's behaviour is the single `StateDef` named by its definition's `exhibitsStates:`; a part with `fmu:` is a plant (REQ-TRS-FMI-001); a part with neither is inert. `connections: [{from: a.out, to: b.in}]` join ports; `bindingConnections: [{from: a.x, to: b.y}]` join variables.
- **Static checks.** `E930` a part's behaviour cannot be resolved or is ambiguous; `E931` a connection end is not a port of the named part, or two ends are both outputs or both inputs; `E932` a `send`/`accept via` names a port the part does not have; `E933` a binding end is not a variable of the part's machine (or plant), or its type or unit dimension differs between the ends; `E934` malformed composite fields; `W936` a sent event is accepted by no connected part, or an accepted event is sent by none (advisory).
- **Message semantics.** Delivery is asynchronous and reliable. A message arrives at the receiving part's port as the event `<port>/<event>` (the form `accept: {payload, via}` already lowers to) with its payload; each message is one run-to-completion step of the receiver. Messages to one receiver are delivered in emission order; the global order of simultaneous work is the order of the parts in the composite, so a run is fully determined. A connection may declare `delay: <duration>` (default zero).
- **Binding semantics.** After every settle, bound values flow from the source to the sink (converted to the sink's unit) and the sink machine settles again, until nothing changes. A cycle that does not converge within 1000 rounds stops the run with an error.
- **Time.** One virtual clock. The next instant is the earliest of: a part's timer, a pending delayed message, a plant communication point. Work due at the same instant is processed to quiescence before the clock moves on; more than 100000 micro-steps at one instant is a livelock error that names the parts involved.
- **Scenarios.** A TestCase whose `subject:` is a composite uses the existing closed vocabulary with instance qualification: `When <part> receives <event> [with …]`, `When <duration> elapses`, `Then <part> is in state <State>`, `Then <part>.<variable> <op> <value>`, `Then <part> sent <event> [via <port>]`.
- **Evidence.** The IR hash of a composite run covers the hashes of every part's machine and the wiring; the trace is one timeline in which every step names its part and every delivered message names its sender, port and send time.
- **Not in v1:** action flows as part behaviour, parallel threads or non-deterministic scheduling, lossy or reordering channels, dynamic part creation, multiplicity greater than one, mutation of one part against composite scenarios (planned).
