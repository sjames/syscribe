---
id: TC-PROJ2-MPS-001
type: TestCase
testLevel: L3
status: active
name: "TC-PROJ2-MPS-001"
verifies:
  - REQ-PROJ2-MPS-001
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-PROJ2-MPS-001
  Scenario: s
    Given x
    Then REQ-PROJ2-MPS-001 holds
```
