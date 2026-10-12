---
type: TestCase
id: TC-QM-001
name: t
status: active
testLevel: L3
subject: Exec::Chain
verifies: [REQ-QM-001, REQ-QM-002]
---

```gherkin
Feature: chain
  Scenario: trip
    Given the chain
    Then the relay opens
```
