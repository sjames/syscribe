---
id: TC-TRS-FMED-006
type: TestCase
testLevel: L3
status: active
name: "Verify the configuration matrix and comparison: row order and depth, collapse and query filtering, differing rows, only-in lists and cell glyphs."
verifies:
  - REQ-TRS-FMED-006
sourceFile: repo:crates/syscribe-server/frontend/test/feature-core.test.mjs
tags:
  - feature-model
  - variability
---

Run with `npm test` in `crates/syscribe-server/frontend/`; the scenarios are *the matrix has a row per shown feature*, *a collapsed subtree is not in the matrix and a query keeps matches with their ancestors*, *the matrix marks the features two compared configurations disagree on*, *comparing two configurations lists what only one selects* and *a cell reads as tick, cross or dot*.

```gherkin
Feature: TC-TRS-FMED-006

  Scenario: the behaviour the requirement states
    Then the tests named above pass
```
