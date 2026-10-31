---
type: PlanningItem
id: PI-BT-PROJ-001
name: "Projection retains elements bound after the projected stage"
status: done
itemType: task
parent: PI-BT-PHASE4-001
evidence:
  - path: "repo:crates/syscribe-model/src/retention.rs"
  - path: "repo:crates/syscribe-model/src/projection.rs"
  - path: "repo:crates/syscribe-model/src/staging.rs"
  - path: "repo:crates/syscribe-model/tests/retention.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-031.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-032.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-033.sh"
achieves: [REQ-TRS-BT-031, REQ-TRS-BT-032, REQ-TRS-BT-033]
tags:
  - variability
  - binding-time
---

Retain and mark variant-retained any element whose `appliesWhen:` depends on a later-bound item; report dependents that rely on its absence; keep unstaged projection byte-identical. Verified by `TC-TRS-BT-031..033`.

Done. Projection at a stage evaluates `appliesWhen:` three-valued (Kleene): a feature with a window extending beyond the Configuration's stage that its chain has not decided is unknown. An element the open features decide is retained and marked `variant_retained`, not dropped; `validate --config` lists each as `I750` and `residual` lists the same set from one `retention::report`; an element that references a retained one yet applies only where it does not is `W752`. An unstaged Configuration takes the original two-valued path (a property test asserts the equality).
