---
type: PlanningItem
id: PI-FMED-001
name: "Viewer and analysis: FeatureModel diagram kind, /features page, analysis API with reasons, overlays, search, collapse, export"
status: done
itemType: feature
achieves: [REQ-TRS-FMED-001]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/feature.rs
  - path: repo:crates/syscribe-model/tests/vis_feature_model.rs
  - path: repo:crates/syscribe-server/tests/feature_model_page.rs
  - path: repo:crates/syscribe-server/frontend/test/feature-core.test.mjs
tags:
  - feature-model
---

See `docs/design/feature-model-editor.md` §4.
