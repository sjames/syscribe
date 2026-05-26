---
type: CalculationDef
name: ThrustToWeightCalc
supertype: Calculations::Calculation
features:
  - name: totalThrustN
    typedBy: ScalarValues::Real
    direction: in
  - name: totalMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: g
    typedBy: ScalarValues::Real
    direction: in
    value: "9.81"
    valueKind: bound
  - name: thrustToWeightRatio
    typedBy: ScalarValues::Real
    direction: out
    isDerived: true
expression: "thrustToWeightRatio = totalThrustN / (totalMassKg * g)"
---

Computes thrust-to-weight ratio for propulsion adequacy verification.
