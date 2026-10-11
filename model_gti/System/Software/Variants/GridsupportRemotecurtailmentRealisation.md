---
type: Part
name: GridsupportRemotecurtailmentRealisation
domain: software
appliesWhen: Features::GridSupport::RemoteCurtailment
satisfies:
  - REQ-GTI-SYS-013
---

Realisation of REQ-GTI-SYS-013 that exists only when `Features::GridSupport::RemoteCurtailment` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-GRIDCTL`.
