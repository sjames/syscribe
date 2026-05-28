---
type: PartDef
name: Sensor
domain: hardware
isAbstract: true
features:
  - name: signalOutputType
    type: ScalarValues::String
  - name: operatingTempMin
    type: ScalarValues::Real
    unit: degC
  - name: operatingTempMax
    type: ScalarValues::Real
    unit: degC
---

Abstract base definition for all engine sensors. Concrete sensor PartDefs
specialize this type (`supertype: System::Sensors::Sensor`) and inherit the
common signal output type and operating temperature range features.

## Extension points

Concrete subtypes must declare their signal-specific features (e.g.,
`teethCount` for VR sensors, `outputRange` for potentiometric sensors).
