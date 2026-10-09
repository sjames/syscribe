---
id: TC-TRS-FTA-005
type: TestCase
testLevel: L3
status: draft
name: "Verify cut-set driven metrics: gate logic changes SPFM/PMHF, W965 on missing DCl, and metrics exits 1 on a failing goal."
verifies:
  - REQ-TRS-FTA-005
---

Verify the metrics roll-up is driven by cut sets and gates the exit code.

```gherkin
Feature: Cut-set driven metrics

  Scenario: OR of two events
    Given OR(a,b) with lambda 1e-6 and DC 0.9 each
    When the tool runs metrics
    Then SPFM is 0.9000 and the goal fails

  Scenario: AND of the same events
    Given AND(a,b) with the same data
    When the tool runs metrics
    Then SPFM is 1.0000, PMHF is 2.000e-10 and the goal passes

  Scenario: Missing latent coverage
    Given one event declares latentDiagnosticCoverage and the other does not
    When the tool validates the model
    Then W965 names the event without it

  Scenario: Exit code
    When metrics runs on the failing and the passing model
    Then it exits 1 and 0 respectively
```
