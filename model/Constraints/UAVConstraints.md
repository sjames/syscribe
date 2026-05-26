---
type: Constraint
name: UAVConstraints
typedBy: UAV::UAVSystem
features:
  - name: checkMass
    type: Constraint
    typedBy: Constraints::MassMarginConstraint
    bindingConnections:
      - left: totalMassKg
        right: UAV::UAVSystem::totalMassKg
  - name: checkBattery
    type: Constraint
    typedBy: Constraints::BatteryEnergyConstraint
    bindingConnections:
      - left: capacityWh
        right: UAV::Power::BatteryPack::capacityWh
      - left: nominalVoltageV
        right: UAV::Power::BatteryPack::nominalVoltageV
---

Composite constraint usage applying all system-level constraints to the UAV top-level element.
