---
id: TC-TRS-FMED-005
type: TestCase
testLevel: L3
status: active
name: "Verify impact analysis: what a feature gates by type and through packages, who selects it, what depends on it, and the summary text."
verifies:
  - REQ-TRS-FMED-005
sourceFile: repo:crates/syscribe-server/tests/feature_model_page.rs
testFunctions:
  - impact_lists_what_a_feature_gates_and_who_selects_it
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-server --test feature_model_page` and `cargo test -p syscribe-model --lib analysis_json` (the engine, including a package gate and the not-found case); the summary text by `frontend/test/feature-core.test.mjs`.

```gherkin
Feature: TC-TRS-FMED-005

  Scenario: the behaviour the requirement states
    Then the tests named above pass
```
