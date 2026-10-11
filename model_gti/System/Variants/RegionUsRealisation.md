---
type: Part
name: RegionUsRealisation
domain: system
appliesWhen: Features::Region::US
satisfies:
  - REQ-GTI-SYS-005
  - REQ-GTI-SYS-042
  - REQ-GTI-SYS-061
---

Realisation of REQ-GTI-SYS-005, REQ-GTI-SYS-042, REQ-GTI-SYS-061 that exists only when `Features::Region::US` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `System::GridTieInverter`.
