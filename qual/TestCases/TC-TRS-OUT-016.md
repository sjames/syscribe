---
id: TC-TRS-OUT-016
type: TestCase
testLevel: L3
status: draft
name: "Verify the n2 interface-matrix command: connection edges appear as named interfaces in the right cells; --allocations adds allocation edges; --format json matches the schema; --format html is a table; --interfaces-only and --depth behave."
verifies:
  - REQ-TRS-OUT-016
---

Verify the `n2` N² interface matrix command against a small wired composite.

```gherkin
Feature: N² interface matrix (§16)

  Scenario: connection edges appear as named interfaces
    Given a composite System with subparts A→B→C wired by IfaceAB and IfaceBC
    When `n2 System` is run
    Then the matrix names IfaceAB and IfaceBC in the correct cells

  Scenario: --allocations adds allocation edges
    Given an allocatedTo edge from PartA to PartC
    When `n2 System --allocations` is run
    Then an allocation edge is shown

  Scenario: --format json matches the schema
    When `n2 System --format json` is run
    Then the output contains a matrix object with kind/name entries

  Scenario: --format html is a self-contained table
    When `n2 System --format html` is run
    Then the output contains an n2-matrix table

  Scenario: --interfaces-only retains the wired elements
    When `n2 System --interfaces-only` is run
    Then the wired interfaces are still shown
```
