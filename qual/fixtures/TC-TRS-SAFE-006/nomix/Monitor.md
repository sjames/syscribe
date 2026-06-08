---
type: PartDef
name: Monitor
domain: software
asilLevel: D
allocatedTo:
  - HostECU
---

A second ASIL D software component. Same integrity tag as SafetyCore, so the sharing on
HostECU is not mixed-criticality and no W034 is raised — even though the check is active
(asilLevel is declared, so the pass is not dormant).
