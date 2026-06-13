---
id: TC-TRS-FMEA-002
type: TestCase
testLevel: L3
status: draft
name: "Verify FMEA entry canonical fields: fmeaSeverity accepted, RPN auto-computed, unknown keys raise E922"
verifies:
  - REQ-TRS-FMEA-002
---

```gherkin
Feature: FMEA entry canonical field vocabulary and unknown-key detection

  Scenario: fmeaSeverity-keyed entry with implicit RPN triggers W903
    Given an FMEASheet entry with fmeaSeverity 9, occurrence 9, detection 9, no explicit rpn, and no recommendedAction
    When the tool validates the model
    Then W903 is raised with the computed RPN 729 in the message

  Scenario: deprecated severity: alias is accepted without diagnostic
    Given an FMEASheet entry with severity 8, occurrence 4, detection 3, no explicit rpn, and no recommendedAction
    When the tool validates
    Then no E922 is raised for the severity key

  Scenario: unknown key in FMEA entry raises E922
    Given an FMEASheet entry with an unrecognised key failureEffect
    When the tool validates
    Then E922 is raised naming failureEffect

  Scenario: template emits fmeaSeverity not severity
    Given the user runs syscribe template FMEASheet
    Then the output contains fmeaSeverity: and does not contain a bare severity: line
```
