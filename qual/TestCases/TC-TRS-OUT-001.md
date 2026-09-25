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

  Scenario: Report title names the model root package (GH #174)
    Given a model whose root _index.md has name: ValidModel
    When the tool is invoked with the model directory
    Then the first line of the report is "# ValidModel Validation Report"
    And the report does not mention UAV

  Scenario: Report title falls back to a neutral heading (GH #174)
    Given a model with no root _index.md
    When the tool is invoked with the model directory
    Then the first line of the report is "# Model Validation Report"
```
