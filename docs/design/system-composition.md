# System-level composition — design

GH #297, requirement `REQ-TRS-SYSCOMP-001`, part of the executable-MBSE epic #272.

## Problem

State machines (#275), action flows (#276) and FMU plants (#291) each execute alone. A real system is several parts, each with behaviour, connected through ports. The model already says how they connect (`features:` parts, port features, `connections:`, `bindingConnections:`), but none of it takes part in execution. This note makes a composite executable.

## Standards basis (SysML v2)

* A part's behaviour is a state usage it *exhibits* (`exhibit state`) — `exhibitsStates:` in the native format.
* Communication is by **send actions** and **accept actions with `via`**: a `send` names a payload and a target (a port); an accept names the payload and the port it arrives on. Messages are asynchronous; the specification leaves queueing and scheduling to the execution semantics, so this note fixes them.
* **Binding connectors** (`bind a = b`) say two features always have the same value; they are not messages.
* **Item/interface flows** over connections carry things between ports. We model the discrete case as messages.

## Model

```
composite Part/PartDef
  features:
    - {name: ctl,   typedBy: Ctl}       # PartDef exhibiting the machine CtlStates
    - {name: ball,  typedBy: BallPlant} # a Part/PartDef with fmu: (plant)
  connections:
    - {from: ctl.cmdOut, to: ball.cmdIn, delay: "2 ms"}   # ports; message channel
  bindingConnections:
    - {from: ball.h, to: ctl.height}                      # variables; value channel
```

* The behaviour of a part is the one `StateDef` in its definition's `exhibitsStates:` (inherited through `supertype`). Zero → inert (still a valid endpoint for static checks); more than one → `E930`.
* A port connection joins an `out`/`inout` port to an `in`/`inout` port. Fan-out (one output, several inputs) delivers to each in connection order.
* The machine side: `effect: {send: Cmd, via: cmdOut, payload: {level: "x + 1"}}` and the existing `accept: {payload: Cmd, via: cmdIn}`. Payload entries are expressions evaluated at send time in the sender's scope. `via` must name a port of the owning part (`E932`). Within one action map `assign:` runs before `send:`, so the payload reports the updated values; a `send` in `initialEffect`, `entryAction` or `exitAction` works the same way.
* Plants are parts with `fmu:`; they will take part through `bindingConnections` only (no messages), using the FMI loop of `fmi-plant-cosimulation.md` with the part's own `plantStep`. The composite carries `plantStep`; `SystemModel::build` resolves each plant's FMU file against the model root, verifies the declared `sha256` and checks every plant binding with the same function as TestCase-level plants (`plant::check_exchange`: causality, type, unit dimension); `System::new` instantiates fresh plants.

## Execution

State per instance: a `Run` (existing), an inbox, and an outbox filled by `send` effects. Global state: the clock, a delay queue of `(due, seq, message)`.

```
quiesce():                                   # all work due at the current instant
  repeat
    for part in declaration order:  settle(part)               # timers/changes due now
    move outboxes → delay queue with due = now + connection.delay, seq increasing
    deliver every queue entry with due <= now, in (due, seq) order:
        receiver.send(port/event, payload)   # one RTC step of the receiver
    propagate bindings (source → sink, unit-converted); settle sinks that changed
  until nothing changed        # guard: 100000 micro-steps → livelock error naming the parts

advance(Δ):
  target = now + Δ
  while now < target:
     t = min(target, earliest timer of any part, earliest delay-queue due, next plant grid point)
     advance every part to t (plants step on their grids, cut at t)
     now = t;  quiesce()
```

Determinism: no threads, no wall clock; every ordering is a total order (declaration order of parts, emission sequence number). Zero delay does not make a message synchronous: it is delivered in the same instant but after the sender's run-to-completion step finished, preserving the run-to-completion rule per part.

## Static checks

`E930` behaviour unresolved/ambiguous · `E931` connection end is not a port of that part, or direction clash · `E932` `send`/`via` names a port the part lacks · `E933` binding end unknown, or type/dimension mismatch · `E934` malformed composite fields (`delay` not a duration, entry not `{from,to}`) · `W936` event sent but accepted by no connected part (or the reverse). All are draft-suppressed where the rest of the behavioural checks are.

## Scenarios

Same Gherkin, closed vocabulary, with instance qualification (`REQ-TRS-SYSCOMP-001`): `When ctl receives Start`, `Then ctl is in state Running`, `Then ctl.height < 0.5 m`, `Then ctl sent Brake via cmdOut`, `When 50 ms elapses`. "Sent" is satisfied by any message the part emitted earlier in the same scenario (the log is per scenario, not windowed); `did not send` is its negation. A part variable is written `<part>.<variable>` in expressions. Plant variables are read through bindings into a machine variable; they are not addressable in expressions.

## Evidence and trace

* Hash of a composite run: BLAKE3 over the canonical JSON of `{parts: [{name, machine}], connections, bindings}` (each part's machine hash, then the wiring).
* Trace: the steps of every part merged by time and tagged `[part]`; the messages are a separate list in the verdict (`messages`: who sent what via which port, when, and when it was delivered). Plant provenance as in the FMI note (`plants` in the verdict).
* Coverage: not yet; `--coverage` is refused for composite subjects.

## Staging

1. IR: `Action::Send` (additive; `skip_serializing_if` keeps old hashes), `send` in the schema checker, `Run` outbox. Tests: IR snapshots unchanged for machines without `send`; outbox contents.
2. Schema/static checks `E930`–`E934`, `W936`; catalogue in all four places, help pages.
3. Engine (`system.rs`): instances, queue, binding propagation, time advance, livelock guard. Property-style tests with deterministic ordering and delay.
4. Scenarios (`sm_scen` extension) and `simulate` for a composite subject; trace and evidence.
5. Plants as parts (done, #299); coverage per part; docs, model-guide page, qualification.

## Deliberately not here

Per-part coverage, mutation against composite scenarios, Action flows as part behaviour, multiplicity > 1, lossy/reordering channels, preemptive or parallel scheduling, mutation against composite scenarios, a graphical timeline. All are additive later.

## Known limits

* `E930` is only raised for composites that have at least one executable part (the checks are dormant otherwise, so existing models see nothing). A part with several `exhibitsStates` is an error in a composite, although SysML allows several exhibited states.
* Ports declared as separate `Port` elements (rather than inline `features:`) and inherited composite wiring (`connections:` from a supertype) are not read.
* `trace()` orders steps of different parts by time only.
