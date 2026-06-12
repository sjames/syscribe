---
id: TC-TRS-MG-017
type: TestCase
testLevel: L3
status: draft
name: "Verify the companion matrices render as 2-D grids: matrix --allocations is a sources×targets ✓ matrix with a gap rollup; trade-study is a Configuration×MoE matrix with a winner."
verifies:
  - REQ-TRS-MG-017
---

```gherkin
Feature: allocation and trade-study reports render as 2-D matrix grids
  Scenario: allocation is a sources × targets matrix
    Given a model with allocation edges and one unallocated source
    When matrix --allocations runs
    Then a sources×targets matrix with ✓ cells is shown
    And the unallocated source is reported as a gap

  Scenario: trade-study is a Configuration × MoE matrix with a winner
    Given a model with a Configuration scored against MoEs
    When trade-study runs
    Then a MoE × Configuration score matrix is shown with the winning configuration marked
```
