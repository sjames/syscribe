---
type: PlanningItem
id: PI-BT-STAGE-001
name: "Stages: bindingStage on Configuration, membership, monotonic chain, closed values"
status: done
itemType: task
parent: PI-BT-PHASE2-001
evidence:
  - path: "repo:crates/syscribe-model/src/staging.rs"
  - path: "repo:crates/syscribe-model/src/validator.rs"
  - path: "repo:crates/syscribe-model/tests/staged_configuration.rs"
  - path: "repo:qual/tests/tc/TC-TRS-BT-011.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-012.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-013.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-014.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-015.sh"
achieves: [REQ-TRS-BT-011, REQ-TRS-BT-012, REQ-TRS-BT-013, REQ-TRS-BT-014, REQ-TRS-BT-015]
tags:
  - variability
  - binding-time
---

Add `bindingStage:`; enforce window membership, monotonic `derivedFrom:` chains, closed ancestor values with narrow-only ranges, and the closed-window-unbound error; keep `W027` for unstaged Configurations. Allocate the new codes and register them in both catalogue files. Verified by `TC-TRS-BT-011..015`.

Done. `staging.rs` owns the chain-level rules and the shared `StageContext`: `E751` unknown `bindingStage:`, `E752` window membership (features there, parameters in `parameter_binding_findings`), `E753` monotonic stages, `E754` closed ancestor values, `E755`/`E756` narrow-only `parameterNarrowing:` (a new optional Configuration field, the mechanism that makes "narrow-only" expressible), `E757` closed-window-unbound in place of `W017`. `W027` stays for unstaged Configurations only. Unstaged models are unchanged.
