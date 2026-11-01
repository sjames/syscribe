---
type: PlanningItem
id: PI-BT-VOCAB-001
name: "Vocabulary: BindingTime type, [bindingTimes], binding-times command"
status: done
itemType: task
parent: PI-BT-PHASE1-001
evidence:
  - path: "repo:crates/syscribe-model/src/binding_time.rs"
  - path: "repo:crates/syscribe/tests/mcp_binding_times.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-001.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-002.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-003.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-004.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-005.sh"
achieves: [REQ-TRS-BT-001, REQ-TRS-BT-002, REQ-TRS-BT-003, REQ-TRS-BT-004, REQ-TRS-BT-005]
tags:
  - variability
  - binding-time
---

Replace the duplicated rank logic in `feature_model.rs`, `validator.rs` and `placeholders.rs` with one `BindingTime` type; add `[bindingTimes]` parsing, the malformed-entry warning, `binding-times`, its MCP tool and the agent-instructions section. Verified by `TC-TRS-BT-001..005`.

Done. `crates/syscribe-model/src/binding_time.rs` owns the vocabulary and the single `BindingTime` type; the validator, feature model and placeholders use it (the duplicated rank logic and `Some(2)` tests are gone). Adds `[bindingTimes]`, `W750`, `binding-times [--json]`, the MCP `binding_times` tool and the agent-instructions section. `E230` now lists the effective vocabulary. Full workspace tests and `TC-TRS-BT-001..005` pass.
