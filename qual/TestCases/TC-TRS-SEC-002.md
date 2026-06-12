---
id: TC-TRS-SEC-002
type: TestCase
testLevel: L3
status: draft
name: "Verify ISO/SAE 21434 risk determination: risk model, E845, W031, W032, and the cyber-risk view."
verifies:
  - REQ-TRS-SEC-002
---

Verify that the tool computes a `ThreatScenario`'s risk from the max
`damageSeverity` of its `DamageScenario`s and its `attackFeasibility`, that an
untreated high/critical-risk threat produces a gateable `W031` while a treated or
goal-addressed one does not, that a `CybersecurityGoal` with too-low `calLevel`
produces `W032`, that an invalid `riskTreatment` produces `E845`, and that
`cyber-risk` (text + `--json`) surfaces the per-threat risk and flag.

```gherkin
Feature: ISO/SAE 21434 risk determination and untreated-threat gate

  Scenario: an untreated high/critical-risk threat produces W031 and validates with no errors
    Given a ThreatScenario with a severe DamageScenario and high attackFeasibility,
      no riskTreatment, and addressed by no CybersecurityGoal
    When the tool validates the model
    Then validation reports zero errors
    And a W031 finding naming the computed level is emitted for that threat

  Scenario: a treated or goal-addressed high-risk threat produces no W031
    Given a high-risk ThreatScenario with riskTreatment: reduce
      and another high-risk ThreatScenario addressed by a CybersecurityGoal
    When the tool validates the model
    Then no W031 finding is emitted for either threat

  Scenario: a CybersecurityGoal with too-low calLevel produces W032
    Given a CybersecurityGoal listing a critical-risk ThreatScenario but declaring CAL1
    When the tool validates the model
    Then a W032 finding naming actual vs expected CAL is emitted for that goal

  Scenario: an invalid riskTreatment produces E845
    Given a ThreatScenario whose riskTreatment is not avoid/reduce/share/retain
    When the tool validates the model
    Then an E845 finding is emitted

  Scenario: W031 is gateable with --deny
    Given a model with an untreated high-risk ThreatScenario
    When the tool validates with --deny W031
    Then the tool exits non-zero

  Scenario: cyber-risk text shows the risk level and untreated flag
    Given the model with the untreated high-risk threat
    When the tool runs cyber-risk
    Then the output shows the threat's computed risk level and an untreated flag

  Scenario: cyber-risk --json carries the per-threat risk fields
    Given the same model
    When the tool runs cyber-risk --json
    Then the output is a valid JSON array whose entries carry
      id, severity, feasibility, risk, treatment, addressed, and flag
```
