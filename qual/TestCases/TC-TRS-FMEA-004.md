---
id: TC-TRS-FMEA-004
type: TestCase
testLevel: L3
status: draft
name: "Verify an FMEA row without id raises E923 and an explicit rpn disagreeing with S×O×D raises W928."
verifies:
  - REQ-TRS-FMEA-004
---

```gherkin
Feature: FMEA row integrity (TC-TRS-FMEA-004)

  Scenario: a row without id raises E923 naming the row
    Given an FMEASheet whose second entries: row has no id:
    When the tool validates the model
    Then E923 is reported on the sheet naming row 2 and its failure mode

  Scenario: an explicit rpn that disagrees with S×O×D raises W928
    Given a row with fmeaSeverity 5, occurrence 4, detection 3 and rpn 100
    When the tool validates the model
    Then W928 names the row, the explicit value 100 and the computed value 60
    And fmea report shows RPN 60 for that row

  Scenario: a consistent or partial rpn raises no W928
    Given a row whose rpn equals S×O×D, and a row with rpn but no detection
    When the tool validates the model
    Then no W928 names either row
```
