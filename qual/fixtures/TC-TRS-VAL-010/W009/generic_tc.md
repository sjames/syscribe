---
id: TC-W009-TRIG-GEN-001
type: TestCase
name: "Generic test file no longer contains the named test"
status: active
testLevel: L3
verifies:
  - REQ-W009-TRIG-001
sourceFile: suite.robot
testFunctions:
  - function: "Acquire When Missing"
    scenario: "generic missing no longer resolves"
---

```gherkin
Feature: W009 generic trigger
  Scenario: generic missing no longer resolves
    Given the robot suite has no such test case
    Then a W009 finding is emitted
```
