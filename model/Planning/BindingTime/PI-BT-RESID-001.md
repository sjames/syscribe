---
type: PlanningItem
id: PI-BT-RESID-001
name: "Residual variability: residual, partial-binding constraints, configure --stage"
status: done
itemType: task
parent: PI-BT-PHASE3-001
evidence:
  - path: "repo:crates/syscribe-model/src/residual.rs"
  - path: "repo:crates/syscribe-model/src/feature_model.rs"
  - path: "repo:crates/syscribe/src/residual.rs"
  - path: "repo:crates/syscribe-model/tests/residual.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-016.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-017.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-018.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-019.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-020.sh"
achieves: [REQ-TRS-BT-016, REQ-TRS-BT-017, REQ-TRS-BT-018, REQ-TRS-BT-019, REQ-TRS-BT-020]
tags:
  - variability
  - binding-time
---

Add `residual`, three-valued `parameterConstraints:` evaluation, `configure --stage`, the staged-chain check in `feature-check --deep`, and stage-aware `validate --config`. Verified by `TC-TRS-BT-016..020`.

Done. `residual --config` (CLI, MCP `residual` tool) groups open items by next-bindable stage and binder and lists closed ones apart; `parameterConstraints:` are evaluated three-valued by interval arithmetic (sound: violated/satisfied hold for every value the open parameters can take), applied to staged Configurations so unstaged models are unchanged; `configure --stage`; `feature-check --deep` `E758` names the earliest Configuration in a chain to close the last valid completion, and a deliberately partial Configuration is no longer judged complete (`E225`; `E219` for a feature still bindable later); `W017` is not raised for a window that opens after the stage.
