---
type: PartDef
name: MPPT Controller
domain: software
satisfies:
  - REQ-GTI-SYS-022
  - REQ-GTI-SW-006
  - REQ-GTI-SYS-023
features:
  - {name: ctl, type: Port, direction: in}
  - {name: stringSamples, type: Port, direction: in}
  - {name: powerRef, type: Port, direction: out}
  - {name: stringData, type: Port, direction: out}
---

One tracker instance per boost stage. Perturb-and-observe with adaptive step; periodic global scan when enabled; see Behavior::MpptTracker.
