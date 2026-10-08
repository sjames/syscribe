---
id: TC-TRS-FMED-007
type: TestCase
testLevel: L3
status: active
name: "Verify scale: a 2,000-feature, 300-constraint model is analysed, configured, explained and drawn within interactive ceilings, and the large-diagram view rules hold."
verifies:
  - REQ-TRS-FMED-007
sourceFile: repo:crates/syscribe-model/tests/feature_scale.rs
testFunctions:
  - two_thousand_features_are_analysed_configured_and_drawn_interactively
  - a_conflict_in_a_large_model_is_still_explained_quickly
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-model --test feature_scale`. In a release build analysis takes about 0.2 s, a configurator click about 0.25 s and the diagram 50 ms; a real browser against a release server paints in about a second and answers a click in about 0.3 s. The roots, one-level expansion and readable-view rules are checked by `frontend/test/feature-core.test.mjs`.

```gherkin
Feature: TC-TRS-FMED-007

  Scenario: the behaviour the requirement states
    Then the tests named above pass
```
