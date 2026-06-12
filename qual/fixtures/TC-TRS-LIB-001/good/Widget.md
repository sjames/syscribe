---
type: PartDef
name: Widget
features:
  - name: mass
    typedBy: ScalarValues::Real
  - name: count
    typedBy: ScalarValues::Integer
  - name: enabled
    typedBy: ScalarValues::Boolean
  - name: label
    typedBy: ScalarValues::String
  - name: n
    typedBy: ScalarValues::Natural
  - name: payload
    typedBy: Base::DataValue
  - name: load
    typedBy: SI::kg
operations:
  - name: isReady
    returnType: ScalarValues::Boolean
    parameters:
      - name: count
        typedBy: ScalarValues::Integer
---
A part exercising every recognised built-in scalar type — including in an operation
returnType/parameter (the W404-prone context) — plus an import-only SI ref on a feature.
