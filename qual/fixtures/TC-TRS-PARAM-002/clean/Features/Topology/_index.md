---
type: FeatureDef
id: FEAT-FX-081
name: Topology
mandatory: true
groupKind: alternative
parameters:
  - name: maxCpus
    type: ScalarValues::Integer
    range: "1..8"
---
CPU topology of the SoC; carries the maxCpus build parameter.
