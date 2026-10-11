---
type: PartDef
name: MPPT Controller
domain: software
satisfies:
  - REQ-GTI-SYS-022
  - REQ-GTI-SW-006
---

One tracker instance per boost stage. Perturb-and-observe with adaptive step; periodic global scan when enabled; see Behavior::MpptTracker.
