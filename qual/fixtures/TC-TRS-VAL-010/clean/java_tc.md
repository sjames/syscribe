---
id: TC-W009-JV-001
type: TestCase
name: "java testFunction resolves in source"
status: active
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: MutexTest.java
testFunctions:
  - function: "com.example.MutexTest#acquireReturnsOk"
    scenario: "java resolves"
---

```gherkin
Feature: java matcher
  Scenario: java resolves
    Given the source defines the test
    Then no W009 is emitted
```
