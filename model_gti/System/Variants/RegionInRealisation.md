---
type: Part
name: RegionInRealisation
domain: system
appliesWhen: Features::Region::IN
satisfies:
  - REQ-GTI-SYS-006
  - REQ-GTI-SYS-043
  - REQ-GTI-SYS-062
---

Realisation of REQ-GTI-SYS-006, REQ-GTI-SYS-043, REQ-GTI-SYS-062 that exists only when `Features::Region::IN` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `System::GridTieInverter`.
