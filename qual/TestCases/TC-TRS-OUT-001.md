---
id: TC-TRS-OUT-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool writes its validation report to stdout in Markdown format."
verifies:
  - REQ-TRS-OUT-001
---

Verify that the tool writes its validation report to stdout in Markdown format.

```gherkin
Feature: Markdown report on stdout

  Scenario: Report is written to stdout, not stderr
    Given a valid model directory
    When the tool is invoked and stdout is redirected to a file
    Then the output file contains a Markdown document
    And stderr contains no finding output

  Scenario: Redirected stdout produces a parseable Markdown file
    Given a valid model with at least one finding
    When stdout is captured to report.md
    Then report.md opens without error in a Markdown renderer
    And the # heading on the first line is present
```
