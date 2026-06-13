---
id: TC-TRS-SEC-008
type: TestCase
name: "TestCase.securityTestMethod validates recognised values; W809 for unknown"
status: active
testLevel: L1
verifies: [REQ-TRS-SEC-008]
---

Verify valid `securityTestMethod` values pass without warning, and an unrecognised value triggers W809.

```gherkin
Feature: securityTestMethod validation (W809)

  Scenario: a recognised method passes
    Given a TestCase with securityTestMethod set to fuzz
    When I validate the model
    Then no W809 warning is reported

  Scenario: an unrecognised method warns
    Given a TestCase with securityTestMethod set to an unrecognised value
    When I validate the model
    Then the output contains "W809"
```
