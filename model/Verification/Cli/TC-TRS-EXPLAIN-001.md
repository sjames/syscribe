---
type: TestCase
id: TC-TRS-EXPLAIN-001
name: "explain-feature names the causes and the fix for requires, excludes, groups and parameter constraints"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/explain_feature.rs
verifies:
  - REQ-TRS-EXPLAIN-001
tags:
  - cli
---

```gherkin
Feature: explaining a feature's state

  Scenario: requires and excludes
    Then the cause names the choice and the constraints, and dropping the listed choice frees the feature

  Scenario: model-level and group causes
    Then a dead feature is a model-level cause agreeing with feature-check, and group cardinality is a cause

  Scenario: conflicts, parameter constraints and surfaces
    Then clashing choices are a conflict, parameter constraints are listed, and CLI, MCP and LSP agree read-only
```
