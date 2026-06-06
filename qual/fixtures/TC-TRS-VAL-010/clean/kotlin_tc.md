---
id: TC-W009-KT-001
type: TestCase
title: "kotlin testFunction resolves in source"
status: draft
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: MutexTest.kt
testFunctions:
  - function: "acquire returns ok when free"
    scenario: "kotlin resolves"
---

```gherkin
Feature: kotlin matcher
  Scenario: kotlin resolves
    Given the source defines the test
    Then no W009 is emitted
```
