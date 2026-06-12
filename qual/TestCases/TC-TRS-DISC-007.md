---
id: TC-TRS-DISC-007
type: TestCase
testLevel: L3
status: draft
name: "Verify list --status/--sil/--json filters and matrix --status/--gaps-only/coverage footer."
verifies:
  - REQ-TRS-DISC-007
---

Verify that `list <type>` gains `--status`, `--sil` (covering both `silLevel:` and `asilLevel:`) and `--json`, and that `matrix` gains `--status`, `--gaps-only` and a per-config + overall coverage-% footer (text and `--json`).

```gherkin
Feature: list/matrix status, SIL & coverage views

  Scenario: list --status narrows by lifecycle status
    Given a model with Requirements in several statuses
    When the tool runs `list Requirement --status draft`
    Then only requirements whose status is draft are listed
    And a requirement with a different status is not listed

  Scenario: list --sil matches a numeric SIL level
    Given a Requirement with silLevel 4
    When the tool runs `list Requirement --sil 4`
    Then that requirement is listed
    And a requirement without silLevel 4 is not listed

  Scenario: list --sil matches an ASIL letter
    Given a Requirement with asilLevel D
    When the tool runs `list Requirement --sil D`
    Then that requirement is listed

  Scenario: list --json emits a JSON array reflecting a filter
    Given a model with Requirements in several statuses
    When the tool runs `list Requirement --status draft --json`
    Then the output is a JSON array
    And every object has status draft

  Scenario: matrix --status restricts rows
    Given a product-line model with requirements in several statuses
    When the tool runs `matrix --status approved`
    Then only approved requirements appear as rows

  Scenario: matrix --gaps-only keeps only rows with a gap
    Given a product-line model with covered and uncovered requirements
    When the tool runs `matrix --gaps-only`
    Then fully-covered and all-N/A rows are dropped
    And at least one gap row remains

  Scenario: matrix prints a coverage footer
    Given a product-line model
    When the tool runs `matrix`
    Then an overall coverage percentage is printed

  Scenario: matrix --json carries a coverage object
    Given a product-line model
    When the tool runs `matrix --json`
    Then the JSON contains a coverage object with perConfig and overall
```
