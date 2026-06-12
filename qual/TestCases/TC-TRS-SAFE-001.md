---
id: TC-TRS-SAFE-001
type: TestCase
testLevel: L3
status: draft
name: Verify that HazardousEvent validation rules E800-E804, E833-E836, and W800 are enforced
verifies:
  - REQ-TRS-SAFE-001
---

Verify that the tool detects all HazardousEvent field-level and cross-reference validation errors and warnings.

```gherkin
Feature: HazardousEvent validation rules

  Scenario: E800 — missing required field triggers error
    Given a HazardousEvent element missing the id field
    When the tool validates the model
    Then at least one E800 finding is emitted

  Scenario: E801 — invalid severity value triggers error
    Given a valid HazardousEvent with severity set to an invalid value
    When the tool validates the model
    Then at least one E801 finding is emitted

  Scenario: E802 — invalid exposure value triggers error
    Given a valid HazardousEvent with exposure set to an invalid value
    When the tool validates the model
    Then at least one E802 finding is emitted

  Scenario: E803 — invalid controllability value triggers error
    Given a valid HazardousEvent with controllability set to an invalid value
    When the tool validates the model
    Then at least one E803 finding is emitted

  Scenario: E804 — id not matching HE-* pattern triggers error
    Given a HazardousEvent with id that does not match the HE-* pattern
    When the tool validates the model
    Then at least one E804 finding is emitted

  Scenario: E833 — invalid consequence value triggers error
    Given a valid HazardousEvent with consequence set to an invalid value
    When the tool validates the model
    Then at least one E833 finding is emitted

  Scenario: E834 — invalid freqExposure value triggers error
    Given a valid HazardousEvent with freqExposure set to an invalid value
    When the tool validates the model
    Then at least one E834 finding is emitted

  Scenario: E835 — invalid avoidance value triggers error
    Given a valid HazardousEvent with avoidance set to an invalid value
    When the tool validates the model
    Then at least one E835 finding is emitted

  Scenario: E836 — invalid demandRate value triggers error
    Given a valid HazardousEvent with demandRate set to an invalid value
    When the tool validates the model
    Then at least one E836 finding is emitted

  Scenario: W800 — unreferenced HazardousEvent triggers warning
    Given a valid HazardousEvent with no SafetyGoal referencing it
    When the tool validates the model
    Then at least one W800 finding is emitted
```
