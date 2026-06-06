---
id: TC-W009-CPP-001
type: TestCase
title: "cpp testFunction resolves in source"
status: draft
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: mutex_test.cpp
testFunctions:
  - function: "MutexTest.AcquireWhenFree"
    scenario: "cpp resolves"
---

```gherkin
Feature: cpp matcher
  Scenario: cpp resolves
    Given the source defines the test
    Then no W009 is emitted
```
