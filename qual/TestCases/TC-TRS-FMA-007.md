---
id: TC-TRS-FMA-007
type: TestCase
testLevel: L3
status: draft
title: "Verify minimal (MUS) unsat-core explanations exclude unrelated constraints."
verifies:
  - REQ-TRS-FMA-007
---

```gherkin
Feature: Minimal unsat-core explanations
  Scenario: explanation is a minimal conflict set
    Given a void model whose conflict is A requires B and A excludes B, plus unrelated features
    When feature-check --deep runs
    Then the E223 explanation names A, B, requires and excludes
    And it does not name the unrelated features
```
