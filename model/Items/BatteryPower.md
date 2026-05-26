---
type: ItemDef
name: BatteryPower
supertype: Items::Item
features:
  - name: voltage
    typedBy: ISQ::VoltageValue
    unit: SI::V
    direction: out
  - name: current
    typedBy: ISQ::ElectricCurrentValue
    unit: SI::A
    direction: out
---

Electrical power item flowing from battery through the power distribution system. Carries instantaneous voltage and current measurements.
