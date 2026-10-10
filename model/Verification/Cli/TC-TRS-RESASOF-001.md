---
type: TestCase
id: TC-TRS-RESASOF-001
name: "--results-as-of evaluates evidence against a retained run"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_as_of.rs
verifies:
  - REQ-TRS-RESASOF-001
tags:
  - cli
---

```gherkin
Feature: results lens

  Scenario: a retained run
    Then validate and matrix evaluate W010 and verdicts against R1 while the latest sidecar says otherwise

  Scenario: errors
    Then an unknown run exits 1 naming the retained runs and the flag works in both spellings
```
