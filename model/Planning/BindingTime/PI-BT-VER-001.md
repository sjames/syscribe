---
type: PlanningItem
id: PI-BT-VER-001
name: "Verification rules and matrix --stage"
status: done
itemType: task
parent: PI-BT-PHASE4-001
evidence:
  - path: "repo:crates/syscribe-model/src/late_safety.rs"
  - path: "repo:crates/syscribe/src/matrix.rs"
  - path: "repo:crates/syscribe-model/tests/retention.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-034.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-035.sh"
achieves: [REQ-TRS-BT-034, REQ-TRS-BT-035]
tags:
  - variability
  - binding-time
---

Add the opt-in, gateable late-bound safety rule and `matrix --stage`. Verified by `TC-TRS-BT-034..035`.

Done. `W753` (opt-in, draft-suppressed, `--deny`-gateable) flags a safety-levelled requirement influenced by a `load`-or-later feature/parameter through its `appliesWhen:` or a placeholder unless a verifying TestCase lists it in the new `coversVariability:` field; `W754` for an entry naming nothing. `matrix --stage S` shows the grid as it can be known at that stage (`?` for undecided cells, not counted as applicable).
