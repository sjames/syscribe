---
id: TC-TRS-FTA-003
type: TestCase
testLevel: L3
status: draft
name: "Verify fault-tree analyze reports cut sets, exact top probability, importance and beta-factor CCF."
verifies:
  - REQ-TRS-FTA-003
---

Verify the cut-set / probability / importance analysis of `fault-tree analyze`.

```gherkin
Feature: Fault-tree analysis

  Scenario: Cut sets and top probability follow the gate logic
    Given OR(AND(a,b),c) with p(a)=0.1, p(b)=0.2, p(c)=0.05
    When the tool runs fault-tree analyze
    Then the cut sets are {c} and {a,b} and the exact top probability is 0.069

  Scenario: JSON output
    Given the same tree
    When the tool runs fault-tree analyze with --json
    Then the document carries cutSets, topProbability and per-event fussellVesely

  Scenario: Beta-factor common-cause failure
    Given two events in ccfGroup PAIR with ccfBeta 0.1
    When the tool runs fault-tree analyze
    Then an order-1 cut set CCF:PAIR is reported, and --no-ccf removes it

  Scenario: Unknown fault tree
    When the tool analyses a non-existent id
    Then it exits non-zero
```
