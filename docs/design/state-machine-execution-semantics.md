# State-machine execution semantics — standards basis

`GUIDE · DESIGN` · Epic GH #272, state machines GH #275 · Requirements: `REQ-TRS-SMEXEC-001`, `REQ-TRS-SMSCHEMA-001`

This note records, for each rule the interpreter follows, **where it comes from**, so that a trace can be
defended and a deviation is never accidental. Source tags: **[KerML]** the SysML v2 / KerML Kernel Semantic
Library text (`Triggers.kerml`, `Observation.kerml`); **[UML]** UML 2.5.1; **[SCXML]** the W3C Recommendation;
**[inferred]** follows from the structure of a source but is not stated in prose — to be confirmed against the
normative text; **[choice]** a decision of this tool where the standards are silent or differ.

## Change triggers (`accept: {when: <condition>}`)

Syscribe's model language is SysML v2, so its change trigger follows the KerML library, not UML and not SCXML.

| Topic | KerML (SysML v2) | UML 2.5.1 | SCXML | Syscribe |
|---|---|---|---|---|
| Meaning | `ChangeSignal`: "sent when the Boolean result of its condition changes from false to true"; `TriggerWhen` starts a monitored observation [KerML] | a change event is generated each time the expression changes from false to true [UML] | no change event; an eventless transition is **level**-triggered (re-fires whenever its condition is true in a stable configuration) | KerML |
| When monitoring starts | when the accept becomes enabled: `TriggerWhen` calls `startObservation` as the accept starts [KerML] | continuous; evaluation timing is a semantic variation point [UML] | — | on entry of the source state |
| Condition already true when monitoring starts | `ObserveChange` waits only "if the result … is false", so the signal is sent at once [KerML, inferred] | strictly no false→true change occurs, so no event (variation point) | — | **the signal is sent at once** [KerML, inferred] |
| How often | one `ObserveChange` performance, one transfer: one signal per observation [KerML, inferred] | every false→true change | every macrostep while true | **once per arming**; re-armed on re-entry of the state |
| Persistence of a sent signal | a sent signal is received by the observer [KerML] | whether it persists until consumed is a variation point | — | delivered and consumed even if the condition is false again by the time it is dispatched [choice, from KerML] |
| Cancelled | `cancelObservation` [KerML] | — | — | when the source state is exited |
| When is the condition evaluated | "wait for it to become true": continuously [KerML] | continuously, variation point [UML] | after every microstep | after every change of any variable and every time advance; skipping evaluation when no variable it reads changed is an optimisation that cannot change a result, because expressions are pure [choice] |

Consequences the interpreter must keep: a `when` that is true on entry fires on that entry's macrostep; a
`when` that stays true does **not** fire again; a `when` consumed by a transition whose `guard` is then false is
**not** re-delivered until the state is re-entered. An **internal** transition re-arms its own trigger after it
fires and then needs a new false→true change [choice — the standards do not say what an internal transition does
to its observation].

## Time triggers (`accept: {after: <duration>}`)

`TriggerAfter(delay)` is `TriggerAt(clock.currentTime + delay)`, evaluated when the accept starts, and a
`TimeSignal` is a change signal whose condition is `clock.currentTime == signalTime` [KerML]. So `after` is the
same mechanism as `when`: armed on entry of the source state, due `delay` later on the (virtual) clock, one
signal, cancelled on exit. A zero delay is due on entry. The clock is the machine's own virtual clock, advanced
only by the scenario [choice].

## Guard

`guard:` is evaluated **when the trigger occurs**, after it was received, against the variables as they are then
[UML, SCXML]. SysML v2's textual notation is open on the relative order of `accept` and `if`
([SYSML2-246](https://issues.omg.org/issues/SYSML2_-246)); we adopt the UML/SCXML reading, and a guard that is
false consumes the trigger.

## Run-to-completion, selection, ordering

As in `REQ-TRS-SMEXEC-001`: microstep/macrostep, eventless work first, then the internal queue, then one external
event [SCXML §3.13 and Appendix D]; deeper states preempt ancestors, then document order [SCXML]; exit, then
transition content, then entry [SCXML, UML]; parallel regions in lock step [SCXML §3.1.3]. A change signal and a
due timer are *internal-queue* events [choice: SCXML has neither; they originate inside the machine].

## Known open points in the standards, and what we do

| Open point | Reference | Syscribe |
|---|---|---|
| order of transitions inside a run-to-completion step | PSSM11-22 | SCXML order, stated above |
| one transition per state per event, multiple guards | [SYSML2-245](https://issues.omg.org/issues/SYSML2_-245) | one trigger, one guard, one effect per transition |
| a transition waiting for substates to finish vs interruptible substates | [KERML11-52](https://issues.omg.org/issues/KERML11-52) | interruptible: a transition of an ancestor can fire while a substate is active (SCXML) |
| `accept`/`if` order | [SYSML2-246](https://issues.omg.org/issues/SYSML2_-246) | guard after trigger |

## What to verify against the normative texts

The rows tagged *inferred* above rest on reading `Observation.kerml` structure (the `not condition` wait step and
the single transfer). They must be confirmed against the published KerML specification (§9.2.12 Triggers and
§9.2.13 Observation) before a conformance claim is made, and against UML 2.5.1 §13.3.3 for the UML column. Until
then the trace states "SysML v2 library semantics (inferred rows pending confirmation)".
