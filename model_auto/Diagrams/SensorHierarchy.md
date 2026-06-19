---
type: Diagram
name: Sensor Hierarchy
diagramKind: BDD
pumlMode: companion
pumlFile: ./SensorHierarchy.puml
generatedBy: claude-sonnet-4-6
shapes:
  sensor-block:
    ref: System::Sensors::Sensor
    kind: block
  cps-block:
    ref: System::Sensors::CrankshaftPositionSensor
    kind: block
  tps-block:
    ref: System::Sensors::ThrottlePositionSensor
    kind: block
  lambda-block:
    ref: System::Sensors::LambdaSensor
    kind: block
edges:
  cps-inherit:
    ref: System::Sensors::CrankshaftPositionSensor
    source: cps-block
    target: sensor-block
    kind: inheritance
  tps-inherit:
    ref: System::Sensors::ThrottlePositionSensor
    source: tps-block
    target: sensor-block
    kind: inheritance
  lambda-inherit:
    ref: System::Sensors::LambdaSensor
    source: lambda-block
    target: sensor-block
    kind: inheritance
---

Sensor specialisation hierarchy. All three concrete sensor types specialise the
abstract `Sensor` base PartDef, inheriting its `signalOutputType` and operating
temperature range features.

![Sensor Hierarchy](./SensorHierarchy.svg)
