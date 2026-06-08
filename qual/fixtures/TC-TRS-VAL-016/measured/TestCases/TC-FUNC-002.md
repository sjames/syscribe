---
id: TC-FUNC-002
type: TestCase
title: "Functional check of logging"
status: active
testLevel: L3
verifies:
  - REQ-NOSIL-002
---
```gherkin
Feature: logging
  Scenario: nominal
    Given logging enabled
    Then entries are written
```
