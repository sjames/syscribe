---
type: Diagram
name: FlightStatesDerivedSM
diagramKind: StateMachine
subject: Behavior::FlightStates
---

State machine diagram **derived from the model** (`REQ-TRS-VIS-018`): the six states of
`Behavior::FlightStates` with their `entry /` and `do /` actions, the initial pseudostate feeding
`disarmed`, and every transition in the machine labelled `accept [guard] / effect` — all read from
the `StateDef`'s own `subStates:` and `transitions:`. Adding a state or a transition to the model
changes the picture; nothing here needs editing.
