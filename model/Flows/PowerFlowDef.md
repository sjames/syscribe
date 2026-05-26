---
type: FlowDef
name: PowerFlowDef
supertype: Flows::FlowConnection
itemType: Items::BatteryPower
ends:
  - name: sourcePort
    typedBy: Interfaces::PowerPortDef
    direction: out
    isEnd: true
  - name: sinkPort
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
    isEnd: true
---

A directed electrical power flow from a power source port to a power consumer port.
