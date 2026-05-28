---
type: PartDef
name: Lambda Sensor
domain: hardware
supertype: System::Sensors::Sensor
features:
  - name: sensorType
    type: ScalarValues::String
  - name: heatingPower
    type: ScalarValues::Real
    unit: W
---

Wideband lambda (oxygen) sensor measuring exhaust-gas oxygen concentration for
closed-loop fuel control.

The signal is used by `System::Software::FuelControl` to maintain stoichiometric
combustion (λ = 1.0 ± 0.005) under steady-state conditions.
