---
id: TC-TRS-OUT-005
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool exits with code 0 when no Error findings are present."
verifies:
  - REQ-TRS-OUT-005
---

Verify that the tool exits with code 0 when no Error findings are present.

```gherkin
Feature: Zero exit on warning-only or clean model

  Scenario: Model with only warnings exits with code 0
    Given a model that triggers W005 warnings but no E-code errors
    When the tool is invoked
    Then the exit code is 0

  Scenario: Clean model with no findings exits with code 0
    Given a model with no validation findings
    When the tool is invoked
    Then the exit code is 0
```
