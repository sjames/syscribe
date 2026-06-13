---
id: TC-TRS-SAFE-010
type: TestCase
testLevel: L3
status: draft
name: "Verify safety-case appends [unknown] verdict footnote when no results sidecar is loaded"
verifies:
  - REQ-TRS-SAFE-010
---

```gherkin
Feature: safety-case [unknown] verdict footnote

  Scenario: footnote appears when no results sidecar is loaded and TestCase leaves exist
    Given a model with SafetyGoal SG-X and a TestCase TC-X that verifies a derived Requirement
    And no results sidecar has been ingested
    When the user runs safety-case
    Then the output contains the line "(verdicts unknown — run `syscribe ingest-results` to populate)"

  Scenario: JSON output includes verdictsUnknown true when no results sidecar loaded
    Given the same model with unknown-verdict TestCase leaves
    When the user runs safety-case --json
    Then the JSON top-level object contains "verdictsUnknown": true

  Scenario: footnote is suppressed when a results sidecar is loaded
    Given a model where a results sidecar has been ingested
    When the user runs safety-case
    Then the output does not contain the verdictsUnknown footnote

  Scenario: JSON omits verdictsUnknown or sets it false when sidecar loaded
    Given the same model with a results sidecar loaded
    When the user runs safety-case --json
    Then the JSON top-level object does not contain "verdictsUnknown": true
```
