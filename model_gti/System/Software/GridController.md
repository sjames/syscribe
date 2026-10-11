---
type: PartDef
name: Grid Controller
domain: software
satisfies:
  - REQ-GTI-SYS-008
  - REQ-GTI-SYS-011
  - REQ-GTI-SYS-012
  - REQ-GTI-SYS-013
features:
  - {name: ctl, type: Port, direction: in}
  - {name: powerRef, type: Port, direction: in}
  - {name: gridSamples, type: Port, direction: in}
  - {name: pwm, type: Port, direction: out}
---

PLL, DC-link and current control, ride-through, volt-var, freq-watt and curtailment.
