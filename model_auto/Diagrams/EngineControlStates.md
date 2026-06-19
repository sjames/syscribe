---
type: Diagram
name: Engine Control State Machine
diagramKind: StateMachine
pumlMode: companion
pumlFile: ./EngineControlStates.puml
subject: System::EngineControlSoftware
shapes:
  s-init:   {ref: "System::EngineControlSoftware", kind: initial}
  s-idle:   {ref: "System::EngineControlSoftware", kind: state, label: "Idle"}
  s-crank:  {ref: "System::EngineControlSoftware", kind: state, label: "Cranking"}
  s-run:    {ref: "System::EngineControlSoftware", kind: state, label: "Running"}
  s-fault:  {ref: "System::EngineControlSoftware", kind: state, label: "FaultDetected"}
  s-stop:   {ref: "System::EngineControlSoftware", kind: state, label: "SafeStop"}
edges:
  e-init:        {source: s-init,  target: s-idle,  kind: transition}
  e-idle-crank:  {source: s-idle,  target: s-crank, kind: transition, label: "ignitionOn"}
  e-crank-run:   {source: s-crank, target: s-run,   kind: transition, label: "rpmAboveThreshold"}
  e-crank-idle:  {source: s-crank, target: s-idle,  kind: transition, label: "timeout / cranking failed"}
  e-run-fault:   {source: s-run,   target: s-fault, kind: transition, label: "safetyFaultDetected [ASIL D]"}
  e-run-idle:    {source: s-run,   target: s-idle,  kind: transition, label: "ignitionOff"}
  e-fault-stop:  {source: s-fault, target: s-stop,  kind: transition, label: "shutdownCommand"}
  e-stop-idle:   {source: s-stop,  target: s-idle,  kind: transition, label: "resetComplete"}
---

![Engine Control State Machine](./EngineControlStates.svg)

Operational states of the Engine Control Software.  The safety monitor triggers
the `FaultDetected` transition from `Running` under any ASIL-D fault; the
`SafeStop` state ensures actuators reach a known-safe position before returning
to `Idle`.
