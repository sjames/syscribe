---
id: TC-W009-CC-001
type: TestCase
title: "c testFunction resolves in source"
status: draft
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: mutex_test.c
testFunctions:
  - function: "test_mutex_acquire"
    scenario: "c resolves"
---

```gherkin
Feature: c matcher
  Scenario: c resolves
    Given the source defines the test
    Then no W009 is emitted
```
