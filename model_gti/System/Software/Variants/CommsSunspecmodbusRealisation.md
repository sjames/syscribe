---
type: Part
name: CommsSunspecmodbusRealisation
domain: software
appliesWhen: Features::Comms::SunSpecModbus
satisfies:
  - REQ-GTI-SYS-050
---

Realisation of REQ-GTI-SYS-050 that exists only when `Features::Comms::SunSpecModbus` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-COMMS`.
