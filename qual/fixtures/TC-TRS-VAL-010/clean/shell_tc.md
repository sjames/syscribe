---
id: TC-W009-SH-001
type: TestCase
name: "shell testFunction resolves in source"
status: active
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: mutex_test.sh
testFunctions:
  - function: "test_mutex_acquire"
    scenario: "shell resolves"
---

```gherkin
Feature: shell matcher
  Scenario: shell resolves
    Given the source defines the test
    Then no W009 is emitted
```
