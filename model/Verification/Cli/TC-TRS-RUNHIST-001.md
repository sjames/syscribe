---
type: TestCase
id: TC-TRS-RUNHIST-001
name: "named runs are retained and diffed"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_history.rs
verifies:
  - REQ-TRS-RUNHIST-001
  - REQ-TRS-AGENTPOL-005
tags:
  - cli
---

```gherkin
Feature: run history

  Scenario: no run id
    Then ingest without --run writes no history

  Scenario: two runs
    Then runs lists both and diff reports regressions, fixed and still failing

  Scenario: replace and errors
    Then re-ingesting a run id replaces it, an unknown id exits 1 and --fail-on-regression gates

  Scenario: the history goes through the write layer
    Then a dry run writes neither the history nor the sidecar, and a protected history is refused with the rule and file named before the sidecar is written
```
