---
type: Part
name: MpptGlobalscanRealisation
domain: software
appliesWhen: Features::Mppt::GlobalScan
satisfies:
  - REQ-GTI-SYS-023
---

Realisation of REQ-GTI-SYS-023 that exists only when `Features::Mppt::GlobalScan` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-MPPT`.
