---
id: TC-TRS-ALLOC-001
type: TestCase
testLevel: L3
status: draft
title: "Verify two allocation forms over one edge model: allocatedTo-on-source clears MG041/MG081 + derives allocatedFrom; type-less legacy features are edges; E503 unresolved; W503 redundant duplicate."
verifies:
  - REQ-TRS-ALLOC-001
---

```gherkin
Feature: allocation supports allocatedTo-on-source and the standalone Allocation element over one edge model
  Scenario: allocatedTo on a W2 function clears MG081
    Given a W2 ActionDef carrying allocatedTo a logical part
    When validate --profile magicgrid is run
    Then MG081 is not raised for that function

  Scenario: allocatedTo on a logical part clears MG041
    Given a logical part carrying allocatedTo a physical part
    When validate --profile magicgrid is run
    Then MG041 is not raised for that part

  Scenario: the allocatedTo edge appears in the matrix
    Given the form-1 model
    When matrix --allocations is run
    Then the function source and the logical target appear

  Scenario: allocatedFrom is derived on the target
    Given the form-1 model
    When show is run on the logical part
    Then it lists the function under a derived allocatedFrom index

  Scenario: a legacy features entry without type is still an edge
    Given a standalone Allocation element whose features entry sets allocatedFrom and allocatedTo but no type
    When validate --profile magicgrid is run
    Then neither MG081 nor MG041 is raised and the edges appear in the matrix

  Scenario: an unresolved allocatedTo raises E503
    Given an element whose allocatedTo names no model element
    When the model is validated
    Then E503 is raised

  Scenario: the same edge in both forms raises the redundancy warning
    Given a function with allocatedTo a part and a standalone Allocation element declaring the same edge
    When the model is validated
    Then W503 is raised
```
