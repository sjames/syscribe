---
type: PlanningItem
id: PI-FMED-004
name: "Editing: semantic edit operations with a validity preview, undo and redo"
status: done
itemType: feature
achieves: [REQ-TRS-FMED-004]
evidence:
  - path: repo:crates/syscribe-model/src/feature_edit.rs
  - path: repo:crates/syscribe-model/tests/feature_edit.rs
  - path: repo:crates/syscribe-server/tests/feature_model_edit.rs
  - path: repo:crates/syscribe-server/frontend/test/feature-core.test.mjs
tags:
  - feature-model
---

See `docs/design/feature-model-editor.md` §4.
