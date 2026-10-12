---
id: TC-TRS-GOALV-001
type: TestCase
testLevel: L3
status: draft
name: "Verify rate-only and demand-mode goal verdicts, plausibility, and the tree-versus-campaign comparison."
verifies:
  - REQ-TRS-GOALV-001
---

```gherkin
Feature: quantitative goal verdicts
  Scenario: rate-only verdict
    Given a SIL 2 goal over an OR of two events of 1e-7 and 2e-7 /h and a 10 000 h mission, without coverage data
    When the tool runs `metrics`
    Then the goal is judged on PFH 2.996e-7 /h, rate only, against the built-in default 1e-6, and passes with unverified rates
  Scenario: demand mode
    Given the same goal with `target: {mode: demand, proofTestInterval: "8760 h"}`
    When the tool runs `metrics`
    Then the figure is PFDavg 1.314e-3 against 1e-2
  Scenario: implausible claimed rate
    Given an event of 1e-4 /h with `rateStatus: verified`
    When the tool runs `validate` and `metrics`
    Then W988 names the event and the verdict is not credible
  Scenario: tree versus campaign
    Given a tree with a cut set the model defeats, one it survives, and an FMEA fault the tree does not have
    When the tool runs `inject --ft --consistency`
    Then the rows are confirmed, conservative or a model gap, and missing from the tree
```
