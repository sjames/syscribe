---
type: PartDef
name: OperatorConsole
supertype: Parts::Part
features:
  - name: displayResolution
    typedBy: ScalarValues::String
    isConstant: true
  - name: telemetryIn
    type: Port
    typedBy: Interfaces::TelemetryPortReceiverDef
    direction: in
---

Operator workstation displaying live telemetry and accepting mission commands. Receives telemetry from the RF link receiver.
