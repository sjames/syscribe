---
id: TC-TRS-FMA-011
type: TestCase
testLevel: L3
status: draft
name: "Verify opt-in proof-evidence emission (DIMACS CNF) for UNSAT findings."
verifies:
  - REQ-TRS-FMA-011
---

```gherkin
Feature: Proof-carrying evidence
  Scenario: --prove emits a re-checkable DIMACS CNF for a void model
    Given a void feature model
    When feature-check --deep --prove <dir> runs
    Then a non-empty, well-formed DIMACS CNF (void.cnf) is written
  Scenario: no proof files without --prove
    Given a void feature model
    When feature-check --deep runs without --prove
    Then no proof files are written
```
