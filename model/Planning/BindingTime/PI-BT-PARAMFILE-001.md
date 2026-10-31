---
type: PlanningItem
id: PI-BT-PARAMFILE-001
name: "Late-bound parameter file: params export, schema, check, staleness"
status: done
itemType: task
parent: PI-BT-PHASE3-001
evidence:
  - path: "repo:crates/syscribe-model/src/params.rs"
  - path: "repo:crates/syscribe/src/params.rs"
  - path: "repo:crates/syscribe-model/tests/params.rs"
  - path: "repo:examples/staged-configuration/README.md"
  - path: "repo:qual/tests/tc/TC-TRS-BT-023.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-024.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-025.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-026.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-027.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-028.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-029.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-030.sh"
achieves: [REQ-TRS-BT-023, REQ-TRS-BT-024, REQ-TRS-BT-025, REQ-TRS-BT-026, REQ-TRS-BT-027, REQ-TRS-BT-028, REQ-TRS-BT-029, REQ-TRS-BT-030]
tags:
  - variability
  - binding-time
---

Add `params export` (YAML keyed by `FEAT-*` id, deterministic, strict by default), `fallback:`, `consumedBy:`, `--schema` and `params check` with the projection-hash staleness warning. Verified by `TC-TRS-BT-023..030`.

Done. `params export` writes the deterministic YAML values file (features and parameters whose window contains the stage, keyed by `FEAT-*` id, stamped with a blake3 `projection-hash` of the whole contract) and, with `--schema`, a JSON Schema (validated by a standard validator in `TC-TRS-BT-028`). It is strict by default and `--allow-open` writes marked placeholders. `params check` validates a file against the same contract (`E760`-`E762`, reusing `E204`-`E206`) and reports staleness as `W751`. `fallback:` is validated (`E759`) and `consumedBy:` is resolved like `implementedBy:` (`W023`) and shown by `trace`.
