---
type: ConnectionDef
name: PowerConnectionDef
supertype: Connections::BinaryConnection
ends:
  - name: source
    typedBy: Interfaces::PowerPortDef
  - name: receiver
    typedBy: Interfaces::PowerPortReceiverDef
    crossFeatures:
      - name: powerFlow
        typedBy: Items::BatteryPower
        crosses: source
        direction: in
---

A binary connection carrying electrical power between a power output port and a power input port.
