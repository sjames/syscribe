---
id: TC-NOSENSOR-001
type: TestCase
testLevel: L3
status: draft
name: "TC-NOSENSOR-001"
verifies:
  - REQ-SENSOR-001
appliesWhen: "not Features::Sensor"
---

```gherkin
Feature: t
  Scenario: s
    Given a
    When b
    Then c
```
