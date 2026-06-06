---
id: TC-TRS-VAR-004
type: TestCase
testLevel: L3
status: draft
title: "Verify the matrix command emits a Requirement x Configuration coverage grid."
verifies:
  - REQ-TRS-VAR-004
---

Verify that `matrix` rows are requirements, columns are the model's `Configuration` elements, and cells classify N/A vs covered vs gap from `appliesWhen:` + selections + `verifies:`; verify `--json` carries `schemaVersion`.

```gherkin
Feature: Requirement x Configuration coverage matrix

  Scenario: columns are the model Configuration elements
    Given a model with configurations CONF-MPS2-WDT-001 and CONF-M0-BASE-001
    When the tool runs matrix --json
    Then the columns are exactly those two configuration ids
    And the document carries a schemaVersion

  Scenario: unconditional requirement is covered in every configuration
    Given REQ-V4-CORE-001 has no appliesWhen and a covering TestCase
    Then its cell is covered in both configurations

  Scenario: conditioned requirement is N/A where its feature is deselected
    Given REQ-V4-WDT-001 has appliesWhen Features::Wdt
    Then its cell is covered in CONF-MPS2-WDT-001 and N/A in CONF-M0-BASE-001

  Scenario: active requirement with no in-config test is a gap
    Given REQ-V4-WDT-002 is active in CONF-MPS2-WDT-001 with no covering TestCase
    Then its cell is a gap in CONF-MPS2-WDT-001 and N/A in CONF-M0-BASE-001
```
