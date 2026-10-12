---
id: TC-TRS-PORTTYPE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify machine events and payloads are checked against the PortDef items of typed ports."
verifies:
  - REQ-TRS-PORTTYPE-001
---

```gherkin
Feature: typed ports
  Scenario: matching typed ports are clean
    Given a sender and a conjugated receiver typed by one PortDef carrying Sample and Trip
    When the tool runs `validate`
    Then no E932 or E935 is reported
  Scenario: each mismatch is reported
    Given an event the type does not carry, an event sent the wrong way, an unknown payload field, a payload of the wrong dimension, an unknown accepted field and an assignment of the wrong kind
    When the tool runs `validate`
    Then each gives E932 or E935
```
