---
id: TC-TRS-CLI-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that --agent-instructions prints the LLM prompt and exits 0 without requiring -m."
verifies:
  - REQ-TRS-CLI-003
---

Verify that --agent-instructions prints the LLM prompt and exits 0 without requiring -m.

```gherkin
Feature: Agent instructions flag

  Scenario: --agent-instructions prints a non-empty prompt to stdout
    When the tool is invoked as: syscribe --agent-instructions
    Then stdout contains a non-empty prompt document
    And the exit code is 0

  Scenario: --agent-instructions does not require -m
    When the tool is invoked as: syscribe --agent-instructions with no -m argument
    Then the tool does not produce an error about a missing model directory
    And the tool exits with code 0

  Scenario: --agent-instructions does not load or validate any model
    When the tool is invoked as: syscribe --agent-instructions
    Then the output does not contain validation findings
```
