---
type: TestCase
id: TC-TRS-TIMING-001
name: "timing reports each phase, and load plus validation stays inside budget and scales linearly"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/perf_budget.rs
verifies:
  - REQ-TRS-TIMING-001
tags:
  - cli
---

```gherkin
Feature: performance measurement

  Scenario: the repository's models
    Then timing reports the phases and the largest real model is inside the budget

  Scenario: scaling
    Then doubling a synthetic model does not more than triple load plus validation time
```
