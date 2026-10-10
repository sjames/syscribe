---
type: TestCase
id: TC-TRS-COVPLAN-001
name: "coverage tree under --plan counts only the plan's tests and fails clearly outside the lens"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_tree.rs
verifies:
  - REQ-TRS-COVPLAN-001
tags:
  - cli
---

```gherkin
Feature: coverage tree lenses

  Scenario: plan lens
    Given a plan whose effective tests cover only part of a tree
    Then the tree under --plan counts only those tests

  Scenario: outside the lens
    Then a root outside the plan exits 1 naming the lens, and an unknown plan exits 1
```
