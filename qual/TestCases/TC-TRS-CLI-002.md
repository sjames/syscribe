---
id: TC-TRS-CLI-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool reports an error to stderr and exits non-zero for invalid model paths."
verifies:
  - REQ-TRS-CLI-002
---

Verify that the tool reports an error to stderr and exits non-zero for invalid model paths.

```gherkin
Feature: Invalid model path handling

  Scenario: Non-existent path produces an error on stderr
    When the tool is invoked as: syscribe -m /nonexistent/path
    Then the exit code is non-zero
    And stderr contains a human-readable error message
    And stdout contains no Markdown report

  Scenario: Path pointing to a file rather than a directory produces an error
    Given /tmp/notadir.md is a regular file, not a directory
    When the tool is invoked as: syscribe -m /tmp/notadir.md
    Then the exit code is non-zero
    And stderr contains an error message

  Scenario: Omitting -m entirely produces an error
    When the tool is invoked with no arguments
    Then the exit code is non-zero
    And stderr or stdout contains usage information
```
