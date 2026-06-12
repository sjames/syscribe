---
id: TC-TRS-CLI-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool accepts the model directory via -m and --model arguments."
verifies:
  - REQ-TRS-CLI-001
---

Verify that the tool accepts the model directory via -m and --model arguments.

```gherkin
Feature: Model path argument

  Scenario: Short form -m accepts the model directory
    Given a valid model directory at /tmp/testmodel
    When the tool is invoked as: syscribe -m /tmp/testmodel
    Then the tool produces a validation report
    And the exit code reflects the model's validity

  Scenario: Long form --model accepts the model directory
    Given a valid model directory at /tmp/testmodel
    When the tool is invoked as: syscribe --model /tmp/testmodel
    Then the tool produces a validation report identical to the -m form
```
