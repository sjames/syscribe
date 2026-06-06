---
id: TC-W009-GEN-001
type: TestCase
title: "generic testFunction resolves in source"
status: draft
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: mutex_test.robot
testFunctions:
  - function: "Acquire When Free"
    scenario: "generic resolves"
---

```gherkin
Feature: generic matcher
  Scenario: generic resolves
    Given the source defines the test
    Then no W009 is emitted
```
