---
type: TestCase
id: TC-UAV-REG-001
name: "Regulatory compliance review confirms EASA Open Category A3 obligations are met"
status: active
testLevel: L2
verifies:
  - REQ-UAV-REG-000
tags:
  - regulatory
  - review
---

Review confirming that mass compliance and all other EASA Open Category A3 obligations are satisfied by verified leaf requirements.

```gherkin
Feature: Regulatory compliance coverage

  Background:
    Given the certification evidence package is complete

  Scenario: Mass compliance is verified
    When the regulatory evidence package is reviewed
    Then REQ-UAV-MASS-001 shall have at least one active verified TestCase
    And the verified mass shall be within the sub-5 kg regulatory boundary
```
