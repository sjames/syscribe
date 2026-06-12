---
id: TC-TRS-VAL-007
type: TestCase
testLevel: L3
status: draft
name: "Verify that Error and Warning severity are reported consistently in the output."
verifies:
  - REQ-TRS-VAL-007
---

Verify that Error and Warning severity are reported consistently in the output.

```gherkin
Feature: Severity consistency across all findings

  Scenario: Model with only errors shows no warnings in the error section
    Given a model that triggers E001 and nothing else
    When the tool output is examined
    Then the finding for E001 appears under the Errors section
    And no W-code findings appear under the Errors section

  Scenario: Model with only warnings shows no errors in the warning section
    Given a model that triggers W005 and nothing else
    When the tool output is examined
    Then the finding for W005 appears under the Warnings section
    And no E-code findings appear under the Warnings section
```
