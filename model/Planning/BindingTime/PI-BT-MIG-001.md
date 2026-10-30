---
type: PlanningItem
id: PI-BT-MIG-001
name: "Migration: regression guard, documentation, worked example"
status: done
itemType: task
parent: PI-BT-001
evidence:
  - path: "repo:docs/model-guide/variability.md"
  - path: "repo:examples/staged-configuration/README.md"
  - path: "repo:prompts/create-model.md"
  - path: "repo:qual/tests/tc/TC-TRS-BT-038.sh"
  - path: "repo:qual/tests/tc/TC-TRS-BT-039.sh"
achieves: [REQ-TRS-BT-038, REQ-TRS-BT-039]
tags:
  - variability
  - binding-time
---

Keep the additive guarantee across every phase; update spec §9.7, `prompts/create-model.md`, `docs/model-guide/variability.md` and both catalogue files; add a staged-configuration example under `examples/`. Verified by `TC-TRS-BT-038..039` and the end-to-end `TC-TRS-BT-000`.

Done. Verified, not assumed: the pre-feature build (the planning commit, unchanged 0.57.0 source) and the current build give identical `validate` findings on all 21 existing models (`model/` and every example); the one model that differs is the new `examples/staged-configuration`, which uses the new fields. `TC-TRS-BT-038` keeps that property in CI. The guide, spec, both catalogue files, the authoring prompt and the help pages describe every new field, code and command (`TC-TRS-BT-039`).
