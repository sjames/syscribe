---
type: PlanningItem
id: PI-FMED-005
name: "Impact panel, configuration diff and matrix"
status: done
itemType: feature
achieves: [REQ-TRS-FMED-005, REQ-TRS-FMED-006]
evidence:
  - path: repo:crates/syscribe-server/frontend/src/feature-main.ts
  - path: repo:crates/syscribe-server/frontend/test/feature-core.test.mjs
  - path: repo:crates/syscribe-model/src/feature_model.rs
  - path: repo:crates/syscribe-server/tests/feature_model_page.rs
tags:
  - feature-model
---

See `docs/design/feature-model-editor.md` §4.
