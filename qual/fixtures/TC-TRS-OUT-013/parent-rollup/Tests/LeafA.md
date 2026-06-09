---
type: TestCase
id: TC-PR-LEAF-001
testLevel: L3
status: active
title: "Verify the first leaf behaviour"
verifies:
  - REQ-PR-LEAF-001
---

```gherkin
Feature: First leaf behaviour
  Scenario: nominal
    Given the system
    Then REQ-PR-LEAF-001 holds
```
