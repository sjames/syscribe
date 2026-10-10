---
type: TestCase
id: TC-TRS-COVTREE-001
name: "coverage tree aggregates leaves, direct tests and verdicts"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_tree.rs
verifies:
  - REQ-TRS-COVTREE-001
tags:
  - cli
---

```gherkin
Feature: coverage tree

  Scenario: mixed tree
    Then the parent shows leaves 1/3 active, 1 planned and the right glyph per node

  Scenario: full coverage
    Then a parent with all leaves verified and a direct test shows the filled glyph

  Scenario: json and errors
    Then --json carries the counts and an unknown root exits 1
```
