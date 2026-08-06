---
type: PlanningItem
id: PI-RTH-TEST-001
name: "Verify RTH behavior end-to-end on hardware"
status: blocked
itemType: task
parent: PI-RTH-001
tags:
  - rth
---

End-to-end hardware-in-the-loop verification of the full return-to-home
behavior, once the controller logic (`Planning::PI-RTH-IMPL-001`) is
complete. `status: blocked` — waiting on the hardware test rig, currently
allocated to another program. A leaf with no `evidence:`, which raises
nothing: `blocked` is not `done`, so `REQ-TRS-PLANITEM-006`'s leaf-evidence
rule does not apply here regardless of evidence content.
