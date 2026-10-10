---
type: TestCase
id: TC-TRS-REQVMODEL-001
name: "V-model columns classify nodes and the layout honours a column assignment without overlap"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQVMODEL-001
tags:
  - server
---

```gherkin
Feature: explorer V-model layout

  Scenario: classification
    Then stakeholder, system, other requirements, architecture, tests and the rest map to columns 0..5

  Scenario: layout
    Then only occupied columns are used, in order, with no overlapping nodes and a header per column

  Scenario: page control
    Then the page has the layout selector
```
