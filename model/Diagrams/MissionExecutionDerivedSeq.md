---
type: Diagram
name: MissionExecutionDerivedSeq
diagramKind: Sequence
subject: Behavior::MissionExecution
---

Sequence diagram **derived from the model** (`REQ-TRS-VIS-021`): the lifelines, messages,
fragments and activation are generated from `Behavior::MissionExecution`'s own sub-actions —
its `checkWeather` `IfAction` becomes an `alt` fragment around the `abortMission` send, whose
`via: controlOut` chain resolves to the part that owns that port, `UAV::Avionics::FlightController`
(the performer of the action). The generator places every lifeline, message and fragment itself,
so the picture needs no layout engine and no pins. Compare `MissionExecutionSeq`, the hand-listed
manifest form of the same interaction.
