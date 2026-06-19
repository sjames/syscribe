---
type: Diagram
name: System Overview
diagramKind: IBD
pumlMode: companion
pumlFile: ./SystemOverview.puml
subject: System::EngineECU
shapes:
  s-ecu:      {ref: "System::EngineECU",                          kind: PartDef}
  s-wdt:      {ref: "System::Hardware::WatchdogTimer",            kind: PartDef, parent: s-ecu}
  s-can-phy:  {ref: "System::Hardware::CANTransceiver",           kind: PartDef, parent: s-ecu}
  s-sw:       {ref: "System::EngineControlSoftware",              kind: PartDef, parent: s-ecu}
  s-tc:       {ref: "System::Software::ThrottleControl",          kind: PartDef, parent: s-sw}
  s-fc:       {ref: "System::Software::FuelControl",              kind: PartDef, parent: s-sw}
  s-sm:       {ref: "System::Software::SafetyMonitor",            kind: PartDef, parent: s-sw}
  s-esm:      {ref: "System::Software::EngineStallMonitor",       kind: PartDef, parent: s-sw}
  s-csm:      {ref: "System::Software::CANSecurityModule",        kind: PartDef, parent: s-sw}
  s-cps:      {ref: "System::Sensors::CrankshaftPositionSensor",  kind: PartDef}
  s-tps:      {ref: "System::Sensors::ThrottlePositionSensor",    kind: PartDef}
  s-lambda:   {ref: "System::Sensors::LambdaSensor",             kind: PartDef}
  s-ta:       {ref: "System::Actuators::ThrottleActuator",        kind: PartDef}
  s-fi:       {ref: "System::Actuators::FuelInjector",            kind: PartDef}
edges:
  e-cps-ecu:  {source: s-cps,    target: s-ecu,     kind: flow}
  e-tps-ecu:  {source: s-tps,    target: s-ecu,     kind: flow}
  e-lam-ecu:  {source: s-lambda, target: s-ecu,     kind: flow}
  e-ecu-ta:   {source: s-ecu,    target: s-ta,      kind: flow}
  e-ecu-fi:   {source: s-ecu,    target: s-fi,      kind: flow}
  e-sm-wdt:   {source: s-sm,     target: s-wdt,     kind: flow}
  e-wdt-ecu:  {source: s-wdt,    target: s-ecu,     kind: flow}
  e-can:      {source: s-ecu,    target: s-can-phy,  kind: binding}
---

![System Overview](./SystemOverview.svg)

High-level block diagram of the Engine ECU system showing the main components
and their relationships. The ECU integrates hardware (Watchdog Timer, CAN
Transceiver) and software (Safety Monitor, Throttle/Fuel Control, CAN Security)
with three sensor inputs and two actuator outputs.
