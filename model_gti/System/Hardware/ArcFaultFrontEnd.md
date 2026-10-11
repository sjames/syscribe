---
type: PartDef
name: Arc-Fault Front End
domain: hardware
silLevel: 2
appliesWhen: Features::Protection::Afci
satisfies:
  - REQ-GTI-HW-004
features:
  - {name: pvTap, type: Port, direction: in}
  - {name: sig, type: Port, direction: out}
---

Wide-band (about 10 kHz–1 MHz) current-signature pickups per tracker input conditioned for the MCU ADC.
