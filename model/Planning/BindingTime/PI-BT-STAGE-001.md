---
type: PlanningItem
id: PI-BT-STAGE-001
name: "Stages: bindingStage on Configuration, membership, monotonic chain, closed values"
status: todo
itemType: task
parent: PI-BT-PHASE2-001
achieves: [REQ-TRS-BT-011, REQ-TRS-BT-012, REQ-TRS-BT-013, REQ-TRS-BT-014, REQ-TRS-BT-015]
tags:
  - variability
  - binding-time
---

Add `bindingStage:`; enforce window membership, monotonic `derivedFrom:` chains, closed ancestor values with narrow-only ranges, and the closed-window-unbound error; keep `W027` for unstaged Configurations. Allocate the new codes and register them in both catalogue files. Verified by `TC-TRS-BT-011..015`.
