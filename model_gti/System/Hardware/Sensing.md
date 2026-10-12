---
type: PartDef
name: Sensing
domain: hardware
silLevel: 2
satisfies:
  - REQ-GTI-SYS-031
  - REQ-GTI-HW-001
  - REQ-GTI-HW-003
  - REQ-GTI-HW-008
  - REQ-GTI-HW-010
features:
  - {name: dcSig, type: Port, direction: in}
  - {name: acSig, type: Port, direction: in, typedBy: Interfaces::SamplePort, isConjugated: true}
  - {name: tempSig, type: Port, direction: in}
  - {name: adcOut, type: Port, direction: out, typedBy: Interfaces::SamplePort}
  - {name: hwVoltage, type: Port, direction: out, typedBy: Interfaces::SamplePort}
---

Grid voltage and current sensing (isolated), DC voltage and per-string current sensing, residual-current sensor, insulation-measurement network and temperature sensors. Feeds the STM32 ADCs with simultaneous sampling.
