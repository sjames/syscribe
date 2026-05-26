---
type: PortDef
name: ControlPortDef
supertype: Ports::Port
features:
  - name: command
    typedBy: Items::ControlCommand
    direction: in
operations:
  - name: arm
    doc: "Query whether the system is currently armed and ready to accept commands."
    isQuery: true
    isAsync: false
    parameters: []
    returnType: ScalarValues::Boolean
  - name: abort
    doc: "Signal an immediate abort of the current command sequence."
    isAsync: true
    parameters: []
---

Port definition for receiving control commands from the flight controller or operator.
