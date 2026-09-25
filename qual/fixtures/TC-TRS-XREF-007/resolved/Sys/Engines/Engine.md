---
type: PartDef
name: Engine
supertype: Parts::Part
features:
  - name: mass
    typedBy: ISQ::MassValue
  - name: rpm
    typedBy: ScalarValues::Real
  - name: shaft
    typedBy: Sys::Engines::Shaft
---
Qualified supertype into the standard library; inline features typed by library and in-model types.
