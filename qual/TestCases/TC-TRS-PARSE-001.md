---
id: TC-TRS-PARSE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool accepts a model root directory path and uses it as the namespace root."
verifies:
  - REQ-TRS-PARSE-001
---

Verify that the tool accepts a model root directory path and uses it as the namespace root.

```gherkin
Feature: Model root initialization

  Scenario: Valid directory is accepted and processed
    Given a directory containing at least one valid .md element file
    When the tool is invoked with that directory via -m
    Then the tool produces a Markdown validation report on stdout
    And the report lists the element count as at least 1

  Scenario: Empty directory produces zero elements
    Given an empty directory with no .md files
    When the tool is invoked with that directory via -m
    Then the report shows 0 total elements
    And the tool exits with code 0
```
