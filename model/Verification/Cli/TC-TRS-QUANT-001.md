---
type: TestCase
id: TC-TRS-QUANT-001
name: "quantities validate and budgets are compared across units"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/quantities.rs
verifies:
  - REQ-TRS-QUANT-001
tags:
  - safety
---

```gherkin
Feature: structured quantities

  Scenario: validity
    Then a bad kind, unit or value is E895 and a good set is clean

  Scenario: budgets
    Then a child chain over its parent's latency and a reaction over the goal's FTTI are W893, mixed units are normalised and fitting chains are silent
```
