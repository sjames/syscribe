---
type: Calculation
name: UAVPerformanceAnalysis
typedBy: UAV::UAVSystem
features:
  - name: calcMass
    type: Calculation
    typedBy: Calculations::TotalMassCalc
    bindingConnections:
      - left: batteryMassKg
        right: UAV::Power::BatteryPack::massKg
      - left: totalMassKg
        right: UAV::UAVSystem::totalMassKg
  - name: calcEndurance
    type: Calculation
    typedBy: Calculations::FlightEnduranceCalc
    bindingConnections:
      - left: capacityWh
        right: UAV::Power::BatteryPack::capacityWh
  - name: calcThrust
    type: Calculation
    typedBy: Calculations::ThrustToWeightCalc
    bindingConnections:
      - left: totalMassKg
        right: UAV::UAVSystem::totalMassKg
---

System-level performance analysis calculation usage applied to the UAV top element.
