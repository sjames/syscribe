---
type: PartDef
name: PowerSystem
supertype: Parts::Part
features:
  - name: battery
    typedBy: UAV::Power::BatteryPack
    multiplicity: "1"
  - name: pdu
    typedBy: UAV::Power::PowerDistributionUnit
    multiplicity: "1"
  - name: mainPowerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
connections:
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: battery.powerOut
      - end: receiver
        binds: pdu.powerIn
bindingConnections:
  - left: pdu.powerOut
    right: mainPowerOut
---

Power system composing battery pack and PDU. Presents a single consolidated power output port to the airframe.
