---
type: CalculationDef
name: TotalMassCalc
supertype: Calculations::Calculation
features:
  - name: airframeMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: batteryMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: propulsionMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: avionicsMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: payloadMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: totalMassKg
    typedBy: ScalarValues::Real
    direction: out
    isDerived: true
expression: "totalMassKg = airframeMassKg + batteryMassKg + propulsionMassKg + avionicsMassKg + payloadMassKg"
---

Calculates total UAV mass as sum of subsystem masses. Used to verify MTOM constraint.
