---
type: PlanningItem
id: PI-BT-WINDOW-001
name: "Windows: bindingTime/latestBindingTime on features and parameters, ordering checks"
status: done
itemType: task
parent: PI-BT-PHASE1-001
evidence:
  - path: "repo:crates/syscribe-model/src/binding_time.rs"
  - path: "repo:crates/syscribe-model/src/feature_model.rs"
  - path: "repo:crates/syscribe-model/tests/binding_windows.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-006.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-007.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-008.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-009.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-010.sh"
achieves: [REQ-TRS-BT-006, REQ-TRS-BT-007, REQ-TRS-BT-008, REQ-TRS-BT-009, REQ-TRS-BT-010]
tags:
  - variability
  - binding-time
---

Add `latestBindingTime:` and feature-level windows; generalise `E229`/`E230` to windows; run ordering under `validate`; keep `TC-TRS-PARAM-004` green. Verified by `TC-TRS-BT-006..010`.

Done. `BindingWindow` (binding_time.rs) reads `bindingTime:`/`latestBindingTime:` off a `FeatureDef` and its parameters; `feature_model::binding_window_findings` is the single checker for `E230` (unknown stage), `E750` (inverted window) and `E229` (a dependent's window closes before its dependency's opens: `derivedFrom:`, `bindTo:`, feature `requires:`), called by both `validate` and `feature-check` so each reports once. Single-stage models behave as before (`TC-TRS-PARAM-004` unchanged).
