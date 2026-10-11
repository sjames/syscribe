---
type: TestCase
id: TC-TRS-ACTSCHEMA-001
name: "the executable action-flow schema validates and malformed uses raise E883/E884/E887/W935"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/act_schema.rs
verifies:
  - REQ-TRS-ACTSCHEMA-001
tags:
  - action-flow
---

```gherkin
Feature: executable action-flow schema

  Scenario: well formed
    Then variables, assignments, decisions with else, loops and timed accepts validate clean

  Scenario: shared codes
    Then variable and assignment errors reuse E883 and E884

  Scenario: wiring
    Then decision, loop and accept errors are E887

  Scenario: expressions
    Then guards and conditions not in the expression language or naming unknown things are W935

  Scenario: dormant for existing models
    Then an action with opaque guards and no variables or assign raises nothing new
```
