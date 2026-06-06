---
id: TC-TRS-TAG-001
type: TestCase
testLevel: L3
status: draft
title: "Verify the generic --tag filter selects by free-text tags without affecting variant logic."
verifies:
  - REQ-TRS-TAG-001
---

Verify that `--tag` filters elements by their free-text `tags:`, that a misspelt tag is not an error, and that tags never change matrix columns or coverage classification.

```gherkin
Feature: Generic tag filter

  Scenario: list --tag selects only tagged elements
    Given REQ-TAG-001 has tag smoke and REQ-TAG-002 does not
    When the tool lists requirements with --tag smoke
    Then REQ-TAG-001 is listed and REQ-TAG-002 is not

  Scenario: an unknown tag is not an error
    Given no element carries the tag nonexistent
    When the tool lists requirements with --tag nonexistent
    Then the tool exits with code 0

  Scenario: matrix --tag filters rows but not columns
    Given a model with two configurations
    When the tool runs matrix --json --tag smoke
    Then only smoke-tagged requirements are rows
    And both configurations remain as columns
```
