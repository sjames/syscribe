---
type: Part
name: ProtectionRapidshutdownRealisation
domain: software
appliesWhen: Features::Protection::RapidShutdown
silLevel: 1
satisfies:
  - REQ-GTI-SW-005
---

Realisation of REQ-GTI-SW-005 that exists only when `Features::Protection::RapidShutdown` is selected. It lives in its own gated element so that the common platform element does not reference requirements that are inactive in other products. Allocated functionally to `the element named SW-SUPERVISOR`.
