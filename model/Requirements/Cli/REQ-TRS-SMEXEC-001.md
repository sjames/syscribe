---
type: Requirement
id: REQ-TRS-SMEXEC-001
name: "A state machine executes deterministically under the SCXML interpretation algorithm applied to the SysML v2 subset"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

`syscribe-model` shall interpret an executable `StateDef` (REQ-TRS-SMSCHEMA-001) with the W3C SCXML interpretation algorithm, restricted to the streamlined SysML v2 state model, with the deviations below (GH #275, part of #272). The interpretation is a pure function of the machine, its variables, the injected events and the virtual time: no wall clock, randomness or environment.

## Behavior

- **Configuration.** The set of active states, one chain per parallel region; the variable store (typed, in SI base units); an internal and an external event queue; the virtual time; and the armed timers (one per active state with an `after` transition, started on entry, cancelled on exit).
- **Start.** Enter the initial states (running `entryAction`, `initialEffect` and `assign` in entry order), then complete the first macrostep.
- **Macrostep and microstep (SCXML §3.13).** A microstep selects an optimal, conflict-free set of enabled transitions, exits their exit set in exit order (`exitAction`), runs the transition content (`effect`, then `assign`) in document order and enters their entry set in entry order (`entryAction`). A macrostep is microsteps until nothing is enabled and the internal queue is empty. Eventless transitions (no trigger) come first, then the internal queue, which carries change and timer signals and `done` events in the order they arose, then one external event. A transition with a deeper source preempts a conflicting one of an ancestor, also across parallel regions. A run error is recorded in the trace as a final `error` step.
- **Selection.** For an event, guards are evaluated against the variables as they stand at the start of the microstep. A transition in a deeper state preempts one in an ancestor; otherwise the first in document order wins; transitions of different parallel regions are taken together in region order. Two enabled transitions of equal priority and different targets are a nondeterminism: document order applies, and in strict mode the run stops with an error naming both.
- **Change triggers follow the SysML v2 / KerML library, not UML and not SCXML eventless transitions** (`docs/design/state-machine-execution-semantics.md`). Entering the source state *arms* an observation of the `when` condition. If the condition is already true the signal is sent at once; otherwise when it changes from false to true. One signal per arming: it is delivered to the machine's internal queue even if the condition is false again by dispatch, is consumed whether or not the transition's guard then holds, and is not sent again until the state is re-entered (an internal transition re-arms and then needs a new false→true change). Leaving the source state cancels the observation. The condition is observed continuously, i.e. after every change of a variable and every time advance.
- **Time.** `after` is the same mechanism as `when` (KerML `TriggerAfter` = `TriggerAt(clock + delay)`; a `TimeSignal` is a change signal on the clock): armed on entry of the source state, due `delay` later, one signal, cancelled on exit; a zero delay is due on entry. The clock only moves when the run advances it. Advancing by a duration fires the due signals in order of due time, ties broken by the state's document order, each at its due time; variables, change triggers and further timers are processed between them.
- **Events.** An event is a name (a payload type, or `via`-port plus payload type as `<port>/<payload>`), optionally with a payload value bound by `as`. An event no transition accepts is *ignored*: recorded in the trace, an error in strict mode.
- **Errors stop the run.** An unbound name, a dimension, type or domain error in a guard or assignment, a payload field that does not exist, or more than the microstep bound (default 1000 per macrostep, to catch livelock) ends the run with the error and its location recorded in the trace; nothing is retried.
- **Trace.** Every microstep is recorded: virtual time, the event, the transitions taken (source → target, trigger, guard value), states exited and entered, actions run, and variables changed (old → new). The trace is deterministic and byte-identical on a re-run.
- **Deviations from SCXML**, to be stated in `docs/model-guide/state-machines.md`: no history, fork/join, `invoke`, `send` to external targets or multi-event transitions (not in the SysML v2 model); typed unit-checked expressions instead of ECMAScript; a virtual clock instead of delayed sends; expression errors stop the run instead of raising `error.execution`; equal-priority conflicts and unhandled events are reported; `doAction` runs to completion on entry.
