---
type: CalculationDef
name: FlightEnduranceCalc
supertype: Calculations::Calculation
features:
  - name: capacityWh
    typedBy: ScalarValues::Real
    direction: in
  - name: avgPowerW
    typedBy: ScalarValues::Real
    direction: in
  - name: enduranceH
    typedBy: ScalarValues::Real
    direction: out
    isDerived: true
expression: "enduranceH = capacityWh / avgPowerW"
---

Estimates flight endurance in hours from battery capacity and average power consumption.
