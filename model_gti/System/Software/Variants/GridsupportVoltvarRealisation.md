---
type: Part
name: GridsupportVoltvarRealisation
domain: software
appliesWhen: Features::GridSupport::VoltVar
satisfies:
  - REQ-GTI-SYS-011
---

Realisation of REQ-GTI-SYS-011 that exists only when `Features::GridSupport::VoltVar` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-GRIDCTL`.
