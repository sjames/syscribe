---
type: PlantDef
name: Lag
parameters:
  - {name: k, value: "1"}
  - {name: tau, value: "200 ms"}
inputs:
  - {name: u}
outputs:
  - {name: y}
states:
  - {name: x, value: "0"}
equations:
  - "der(x) = k * u - x"
  - "y := x"
---

A derivative of the wrong dimension.
