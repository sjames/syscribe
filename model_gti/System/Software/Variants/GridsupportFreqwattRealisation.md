---
type: Part
name: GridsupportFreqwattRealisation
domain: software
appliesWhen: Features::GridSupport::FreqWatt
satisfies:
  - REQ-GTI-SYS-012
---

Realisation of REQ-GTI-SYS-012 that exists only when `Features::GridSupport::FreqWatt` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-GRIDCTL`.
