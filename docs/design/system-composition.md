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
* Plants are parts with `fmu:`; they take part through `bindingConnections` only (no messages), using the FMI loop of `fmi-plant-cosimulation.md` with the part's own `plantStep` (default: the composite's `plantStep`).

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

Same Gherkin, closed vocabulary, with instance qualification (`REQ-TRS-SYSCOMP-001`): `When ctl receives Start`, `Then ctl is in state Running`, `Then ball.h < 0.5 m`, `Then ctl sent Brake via cmdOut`, `When 50 ms elapses`. "Sent" is satisfied by a message emitted since the previous `Then`-checked point of that part (a per-part sent log, cleared when the scenario ends). Plants are exposed as `<part>.<variable>` read-only.

## Evidence and trace

* Hash of a composite run: BLAKE3 over the canonical JSON of `{parts: [{name, machineHash}], connections, bindings}`.
* Trace: one timeline; each step carries `part`, and message steps carry `from`, `port`, `sent`, `delivered`. Plant provenance as in the FMI note.
* Coverage: per part (existing coverage structures), reported per part and aggregated.

## Staging

1. IR: `Action::Send` (additive; `skip_serializing_if` keeps old hashes), `send` in the schema checker, `Run` outbox. Tests: IR snapshots unchanged for machines without `send`; outbox contents.
2. Schema/static checks `E930`–`E934`, `W936`; catalogue in all four places, help pages.
3. Engine (`system.rs`): instances, queue, binding propagation, time advance, livelock guard. Property-style tests with deterministic ordering and delay.
4. Scenarios (`sm_scen` extension) and `simulate` for a composite subject; trace and evidence.
5. Plants as parts; coverage per part; docs, model-guide page, qualification.

## Deliberately not here

Action flows as part behaviour, multiplicity > 1, lossy/reordering channels, preemptive or parallel scheduling, mutation against composite scenarios, a graphical timeline. All are additive later.
