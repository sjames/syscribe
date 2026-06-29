---
type: TestCase
id: TC-FXPLAN-001
name: "Draft test for the planned requirement"
status: draft
testLevel: L2
verifies:
  - REQ-FXPLAN-001
---

```gherkin
Feature: Planned verification
  Scenario: a draft test is planned, not done
    Given the fixture model
    Then REQ-FXPLAN-001 is linked only by this draft test
```
