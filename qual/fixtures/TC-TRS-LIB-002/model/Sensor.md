---
type: PartDef
name: Sensor
features:
  - name: mass
    typedBy: ISQ::MassValue
    unit: SI::kilogram
  - name: thrust
    typedBy: ISQ::ForceValue
    unit: SI::newton
  - name: exotic
    typedBy: ISQ::WibbleValue
  - name: typo
    typedBy: ScalarValues::Flota
operations:
  - name: report
    returnType: ScalarValues::Boolean
    parameters:
      - name: m
        typedBy: ISQ::MassValue
      - name: x
        typedBy: ISQ::FlibbleValue
  - name: cost
    returnType: ScalarValues::Real
---
A part using recognised ISQ types (clean) plus an unrecognised ISQ member (lenient),
and operation signatures that should not raise W404 for recognised ISQ.
