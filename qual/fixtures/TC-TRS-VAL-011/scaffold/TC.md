---
id: TC-SCAF-FIX-001
type: TestCase
title: "Scaffold fixture: one scenario missing"
status: draft
testLevel: L3
verifies:
  - REQ-SCAF-001
testFunctions:
  - function: "m::tests::case_a"
    scenario: "Case A passes"
  - function: "m::tests::case_b"
    scenario: "Case B passes"
---

```gherkin
Feature: Scaffold fixture
  Scenario: Case A passes
    Given a
    When b
    Then c
```
