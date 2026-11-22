---
type: Requirement
id: REQ-TRS-ACTSEND-001
name: "A SendAction inside an activity run by a state machine is emitted as a message of that machine"
status: implemented
reqDomain: software
reqClass: system
tags:
  - execution
  - action-flow
  - state-machine
  - composition
amendments:
  - date: 2026-11-22
    by: claude
    reason: 'GH #509/#516: W936 follows SendActions inside effect ActionDefs, and W937 reports a referenced sends-only ActionDef that has not opted in with variables: []; the opt-in rule itself is unchanged'
    kind: clarification
---

When a state machine runs an executable `ActionDef` as an `entryAction`, `doAction`, `exitAction`, `initialEffect` or transition `effect` (REQ-TRS-ACTEXEC-001, "From a state machine"), every `SendAction` that activity executes shall be emitted as a message of the machine, exactly as a `send:` effect written on the machine itself is (REQ-TRS-SMEXEC-001, REQ-TRS-SYSCOMP-001). Today the activity runs and its sends are discarded without any diagnostic, so a part whose behaviour is written as an activity cannot talk to the parts it is connected to.

## Behavior

- **Opt-in is unchanged.** An `ActionDef` is executable when it declares `variables:` or uses `assign:` (REQ-TRS-ACTSCHEMA-001). A `SendAction` alone does not make it executable, so a descriptive action that draws a send keeps being checked and rendered as before. An activity that only sends opts in with `variables: []`. A machine's reference to an `ActionDef` that is not executable stays a recorded reference and emits nothing.
- **Emission.** Each `SendAction` that fires during the activity's run produces one message in the machine's outbox, in firing order, after the sends the machine made earlier in the same microstep and before the ones it makes later. The message carries the event name (`payload:`), the port (`via:`), and the payload values as typed, unit-checked values evaluated when the node fired, not when the activity finished.
- **Nesting.** A `SendAction` in a `PerformAction` callee, which a standalone run of the caller also records in its own list of sent messages, in a loop body (once per iteration) or in an `if` branch is emitted the same way. A `SendAction` that does not fire (an untaken branch, a decision arm whose guard is false) emits nothing.
- **Trace.** The microstep's `actions` list records each emitted message with the same text a machine-level `send:` produces (`send <event> via <port> (<name>=<value>, …)`), in order with the activity's own `executed` note.
- **Composition.** In a composite (REQ-TRS-SYSCOMP-001) the message leaves through the part's port and is delivered along the connection, with its delay, exactly like a machine-level send. A part with no connection on that port records it as sent to nothing.
- **Failure.** If the activity ends in an error, or waits for an event (REQ-TRS-ACTEXEC-001, "a behaviour must complete"), the run stops as before and the messages that activity had already produced are not emitted.
- **Diagnostics.** The composite check `W936` counts a `SendAction` inside an executable effect `ActionDef` (and its callees) as a sender of that message. A machine that references an `ActionDef` containing a `SendAction` that is not executable gets `W937` naming it and the `variables: []` opt-in, so the silence is explained (GH #509, #516).
- **No change otherwise.** A machine or activity that contains no `SendAction` behaves and traces exactly as before, and a machine-level `send:` is unaffected.
