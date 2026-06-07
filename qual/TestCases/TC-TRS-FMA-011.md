---
id: TC-TRS-FMA-011
type: TestCase
testLevel: L3
status: draft
title: "Verify opt-in DRAT proof emission for UNSAT findings."
verifies:
  - REQ-TRS-FMA-011
---

```gherkin
Feature: DRAT proof-carrying evidence
  Scenario: --prove emits DIMACS + DRAT for a void model
    Given a void feature model
    When feature-check --deep --prove <dir> runs
    Then a non-empty DIMACS CNF and a DRAT proof file are written
  Scenario: no proof files without --prove
    Given a void feature model
    When feature-check --deep runs without --prove
    Then no proof files are written
```
