---
type: TestCase
id: TC-PR-LEAF-002
testLevel: L3
status: active
title: "Verify the second leaf behaviour"
verifies:
  - REQ-PR-LEAF-002
---

```gherkin
Feature: Second leaf behaviour
  Scenario: nominal
    Given the system
    Then REQ-PR-LEAF-002 holds
```
