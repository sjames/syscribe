---
type: Requirement
id: REQ-TRS-PUML-020
name: "BDD diagramKind maps to PlantUML class diagram with block stereotypes"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

For `diagramKind: BDD`, the generator shall emit a PlantUML `@startuml` class diagram.
Each shape with `kind: PartDef` becomes `class "Name" <<part def>>`, `kind: Part` becomes
`class "Name" <<part>>`. Other kinds fall back to `class "Name" <<kind>>`. Edge kinds:
`composition` → `*--`, `usage` → `..>`, `generalization`/`specialization` → `<|--`,
unknown → `-->`. The diagram is wrapped with `hide empty members` and a `skinparam` block
for consistent styling.

## Example output

```plantuml
@startuml
hide empty members
skinparam classBackgroundColor #FEFECE
skinparam classBorderColor #A80036

class "Vehicle System" as s_vehiclesystem <<block>>
class "Powertrain" as s_powertrain <<part def>>
class "Engine" as s_engine <<part def>>

s_vehiclesystem *-- s_powertrain
s_powertrain *-- s_engine
@enduml
```

## Node alias derivation

The PlantUML node alias is the shape key from the `shapes:` map, sanitised for PlantUML
compatibility as described in REQ-TRS-PUML-026.
