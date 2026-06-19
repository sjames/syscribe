---
type: Diagram
name: FlightStatesMachineD
diagramKind: StateMachine
svgMode: companion
svgFile: ./FlightStatesMachineD.svg
pumlMode: companion
pumlFile: ./FlightStatesMachineD.puml
subject: Behavior::FlightStates
shapes:
  s-initial: {ref: "Behavior::FlightStates", kind: initial}
  s-disarmed: {ref: "Behavior::FlightStates::disarmed", kind: state}
  s-armed: {ref: "Behavior::FlightStates::armed", kind: state}
  s-takingOff: {ref: "Behavior::FlightStates::takingOff", kind: state}
  s-flying: {ref: "Behavior::FlightStates::flying", kind: state}
  s-landing: {ref: "Behavior::FlightStates::landing", kind: state}
  s-fault: {ref: "Behavior::FlightStates::fault", kind: state}
edges:
  e-init: {source: s-initial, target: s-disarmed, kind: transition}
  e-arm: {source: s-disarmed, target: s-armed, kind: transition}
  e-takeoff: {source: s-armed, target: s-takingOff, kind: transition}
  e-armfault: {source: s-armed, target: s-fault, kind: transition}
  e-fly: {source: s-takingOff, target: s-flying, kind: transition}
  e-tofault: {source: s-takingOff, target: s-fault, kind: transition}
  e-land: {source: s-flying, target: s-landing, kind: transition}
  e-flyfault: {source: s-flying, target: s-fault, kind: transition}
  e-disarm: {source: s-landing, target: s-disarmed, kind: transition}
  e-lndfault: {source: s-landing, target: s-fault, kind: transition}
  e-recover: {source: s-fault, target: s-disarmed, kind: transition}
---

![FlightStatesMachineD](./FlightStatesMachineD.svg)

State machine diagram for the UAV flight states lifecycle, showing normal operational transitions from disarmed through takeoff, flight, and landing, as well as fault transitions from any active state and recovery back to disarmed.
