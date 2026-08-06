---
type: TestCase
id: TC-DEMO-002
name: "Verifies a native Requirement (regression)"
status: active
testLevel: L2
verifies:
  - REQ-DEMO-NATIVE-001
---

```gherkin
Feature: verify a native Requirement
  Scenario: native Requirement target still works
    Given a native Requirement
    When the model is validated
    Then no dangling-reference or wrong-type finding is raised
```
