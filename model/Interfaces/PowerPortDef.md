---
type: PortDef
name: PowerPortDef
supertype: Ports::Port
features:
  - name: voltage
    typedBy: ISQ::VoltageValue
    direction: out
    unit: SI::V
  - name: current
    typedBy: ISQ::ElectricCurrentValue
    direction: out
    unit: SI::A
---

Port definition for electrical power output. A component supplying power exposes this port.
