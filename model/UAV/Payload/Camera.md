---
type: PartDef
name: Camera
supertype: Parts::Part
features:
  - name: resolutionMp
    typedBy: ScalarValues::Real
    isReadonly: true
    isConstant: true
  - name: frameRateFps
    typedBy: ScalarValues::Integer
    isReadonly: true
    isConstant: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Electro-optical camera payload. Resolution and frame rate are fixed hardware parameters. Receives power from the payload bay.
