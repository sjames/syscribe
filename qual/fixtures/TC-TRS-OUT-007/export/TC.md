---
id: TC-EXP-001
type: TestCase
title: "Verify export"
status: active
testLevel: L3
verifies:
  - REQ-EXP-001
---

```gherkin
Feature: export
  Scenario: basic
    Given a model
    Then it exports
```
