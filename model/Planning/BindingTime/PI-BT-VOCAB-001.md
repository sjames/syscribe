---
type: PlanningItem
id: PI-BT-VOCAB-001
name: "Vocabulary: BindingTime type, [bindingTimes], binding-times command"
status: todo
itemType: task
parent: PI-BT-PHASE1-001
achieves: [REQ-TRS-BT-001, REQ-TRS-BT-002, REQ-TRS-BT-003, REQ-TRS-BT-004, REQ-TRS-BT-005]
tags:
  - variability
  - binding-time
---

Replace the duplicated rank logic in `feature_model.rs`, `validator.rs` and `placeholders.rs` with one `BindingTime` type; add `[bindingTimes]` parsing, the malformed-entry warning, `binding-times`, its MCP tool and the agent-instructions section. Verified by `TC-TRS-BT-001..005`.
