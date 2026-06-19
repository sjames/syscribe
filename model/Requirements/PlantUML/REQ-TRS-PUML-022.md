---
type: Requirement
id: REQ-TRS-PUML-022
name: "StateMachine diagramKind maps to PlantUML state diagram"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

For `diagramKind: StateMachine`, the generator shall emit a PlantUML `@startuml` state
diagram. Shapes with `kind: initial` are not emitted as named states; instead, any
`transition` edge whose source is the initial shape is emitted as `[*] --> target_id`.
Shapes with `kind: state` are emitted as `state "Name" as id`. All other `transition`
edges are emitted as `source_id --> target_id`. The diagram title is set to the element's
`name`.

## Transition labels

When a `transition` edge carries a non-empty `label` field, the emitted line shall include
it as a guard/trigger annotation:

```
source_id --> target_id : label
```

When no `label` is present the annotation is omitted.

## Example output

```plantuml
@startuml
title Scheduler State Machine

state "Idle" as s_idle
state "Running" as s_running
state "Fault" as s_fault

[*] --> s_idle
s_idle --> s_running : start
s_running --> s_idle : stop
s_running --> s_fault : error
@enduml
```
