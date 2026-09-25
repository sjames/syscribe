---
type: PartDef
name: Controller
domain: software
supertype: Integration::Lib::Housing
satisfies: [Integration::Brakes::REQ-BRK-001]
features:
  - name: unit
    typedBy: Integration::Brakes::BrakeUnit
---
Local controller reusing mounted peer definitions.
