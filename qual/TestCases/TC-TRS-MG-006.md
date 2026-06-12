---
id: TC-TRS-MG-006
type: TestCase
testLevel: L3
status: draft
name: "Verify the allocation matrix view: rows=sources, cols=targets, allocated cells; unallocated/unused rollup; mg_layer partition; flat fallback; --json."
verifies:
  - REQ-TRS-MG-006
---

```gherkin
Feature: allocation matrix view
  Scenario: allocations render as a matrix
    Given a model with Allocation elements from sources to targets
    When matrix --allocations is run
    Then every source is a row, every target is a column, and allocated cells are marked

  Scenario: the rollup reports gaps
    Given a source with no allocation and a target never allocated to
    When matrix --allocations is run
    Then the rollup reports the unallocated source and the unused target

  Scenario: mg_layer partitions logical to physical
    Given parts marked mg_layer logical and physical
    When matrix --allocations is run
    Then the view separates logical sources from physical targets

  Scenario: flat fallback without mg_layer
    Given a model with allocations and no mg_layer anywhere
    When matrix --allocations is run
    Then the flat Allocation-derived matrix is produced

  Scenario: the matrix emits JSON
    When matrix --allocations --json is run
    Then a JSON grid with rows, columns, cells and rollup is produced
```
