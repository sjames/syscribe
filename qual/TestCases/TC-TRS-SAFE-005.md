---
id: TC-TRS-SAFE-005
type: TestCase
testLevel: L3
status: draft
name: Verify SPFM/LFM/PMHF computation, ASIL/SIL gating (W033), diagnosticCoverage range (E846), and the metrics command
verifies:
  - REQ-TRS-SAFE-005
---

Verify that the tool computes the ISO 26262-5 §8–9 hardware architectural metrics per
`SafetyGoal`, gates them against the goal's target with W033, enforces the diagnosticCoverage
range with E846, and reports the metrics through the `metrics` command.

```gherkin
Feature: Quantitative HW safety metrics (SPFM / LFM / PMHF)

  Scenario: W033 fires on a goal that misses its ASIL target
    Given a model with SafetyGoal SG-MET-001 (ASIL D) whose fault-tree events yield SPFM 0.945 and PMHF 1.1e-8
    When the tool validates the model
    Then at least one W033 finding is emitted

  Scenario: W033 does not fire on a goal that meets its ASIL target
    Given the same model where SafetyGoal SG-MET-002 (ASIL B) yields SPFM 0.99 and PMHF 1e-10
    When the tool validates the model
    Then no W033 finding names SG-MET-002

  Scenario: metrics text output reports SPFM and a fail verdict
    Given the same model
    When the tool runs the metrics command in text mode
    Then the SG-MET-001 row shows SPFM ~0.945 and a fail verdict

  Scenario: metrics --json output carries spfm, pmhf, and pass fields
    Given the same model
    When the tool runs the metrics command with --json
    Then the JSON array contains spfm, pmhf, and pass for SG-MET-001

  Scenario: E846 fires on diagnosticCoverage out of range
    Given a FaultTreeEvent with diagnosticCoverage set to 1.5
    When the tool validates the model
    Then at least one E846 finding is emitted

  Scenario: --deny W033 makes validation exit non-zero
    Given the model where SG-MET-001 misses its target
    When the tool validates with --deny W033
    Then the tool exits with a non-zero status
```
