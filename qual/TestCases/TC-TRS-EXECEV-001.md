---
id: TC-TRS-EXECEV-001
type: TestCase
testLevel: L3
status: draft
name: "Verify requirement-coverage, exec-coverage and @verifies scenario claims."
verifies:
  - REQ-TRS-EXECEV-001
---

```gherkin
Feature: executable evidence
  Scenario: the evidence of each requirement and element is reported
    Given a controller machine carried by an architecture element, a TestCase whose two scenarios each claim one requirement, a descriptive TestCase and an inspection requirement
    When the tool runs `validate`, `requirement-coverage` and `exec-coverage`
    Then neither W994 nor W995 is reported
    And the two claimed requirements are executable, the bench-tested one descriptive-only, the untested one unverified and the inspected one not applicable
    And the controller element is executable with two scenarios, the other element not modelled
    And a missed gate exits 2
  Scenario: tag defects are reported
    Given a scenario tagged with a requirement its TestCase does not verify
    When the tool runs `validate`
    Then W994 names the claim and W995 names the requirement no scenario claims
```
