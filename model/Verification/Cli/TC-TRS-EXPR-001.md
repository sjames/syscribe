---
type: TestCase
id: TC-TRS-EXPR-001
name: "the expression language evaluates, checks dimensions and fails with typed errors"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/expr.rs
verifies:
  - REQ-TRS-EXPR-001
tags:
  - cli
---

```gherkin
Feature: typed expression language

  Scenario: arithmetic, logic and precedence
    Then operators follow their precedence, and and/or short-circuit

  Scenario: units
    Then literals carry scale and dimension, mixed units add, incompatible dimensions are errors

  Scenario: names and enumerations
    Then dotted and qualified names resolve from the environment, unbound names are errors, qualified unbound names are enum literals

  Scenario: eval command
    Then eval prints SI values, converts with --unit, emits JSON and exits 1 on errors
```
