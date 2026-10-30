---
type: PlanningItem
id: PI-BT-PROV-001
name: "Provenance: source and stage of every effective value"
status: done
itemType: task
parent: PI-BT-PHASE2-001
evidence:
  - path: "repo:crates/syscribe-model/src/provenance.rs"
  - path: "repo:crates/syscribe/src/query.rs"
  - path: "repo:crates/syscribe-model/tests/staged_configuration.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-021.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-022.sh"
achieves: [REQ-TRS-BT-021, REQ-TRS-BT-022]
tags:
  - variability
  - binding-time
---

Compute the source and stage of each effective value and surface it in `show`, `configure` and the JSON outputs. Verified by `TC-TRS-BT-021..022`.

Done. `provenance.rs` computes the source (own, inherited from a named ancestor, subConfigurations tier, bindTo, derived expression, fixed, default, abstract-follows) and stage of every effective selection and parameter value. Surfaced by `show <Configuration>` and `configure` (text table and `provenance` in `--json`); `residual` and `params export` will carry the same object when they land in phase 3.
