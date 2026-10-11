---
type: PartDef
name: Power Stage
domain: hardware
satisfies:
  - REQ-GTI-SYS-037
  - REQ-GTI-HW-006
features:
  - {name: boostCount, typedBy: ScalarValues::Integer}
  - {name: dcLinkVoltageV, typedBy: ScalarValues::Integer}
  - {name: switchingFrequencyKhz, typedBy: ScalarValues::Integer}
  - {name: dcIn, type: Port, direction: in}
  - {name: acOut, type: Port, direction: out}
  - {name: gate, type: Port, direction: in}
  - {name: hwDisable, type: Port, direction: in}
  - {name: tempOut, type: Port, direction: out}
---

Per-tracker interleaved boost converters (SiC MOSFETs, 1200 V) feeding a shared DC link of about 800 V, followed by a three-level T-type three-phase bridge. Switching frequency 16–48 kHz from the STM32 HRTIM. A comparator-based overcurrent trip turns off all gate drivers in hardware.
