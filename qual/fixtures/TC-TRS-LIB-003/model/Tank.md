---
type: PartDef
name: Tank
features:
  - name: massOk
    typedBy: ISQ::MassValue
    unit: SI::kilogram
  - name: forceOk
    typedBy: ISQ::ForceValue
    unit: SI::newton
  - name: massBareOk
    typedBy: ISQ::MassValue
    unit: kg
  - name: massVsLength
    typedBy: ISQ::MassValue
    unit: SI::metre
  - name: forceVsPower
    typedBy: ISQ::ForceValue
    unit: SI::watt
  - name: unitUnrecognised
    typedBy: ISQ::MassValue
    unit: USD
  - name: typeNotQuantity
    typedBy: ScalarValues::Real
    unit: SI::kilogram
---
Dimensional consistency: two mismatches (mass≠length, force≠power); the rest clean or
lenient (unrecognised unit / non-quantity type).
