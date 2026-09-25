---
id: TC-TRS-ALLOC-003
type: TestCase
testLevel: L3
status: draft
name: "Verify a features-form allocation on a non-Allocation element raises W930 and contributes no allocation edge, while the same entry on a type: Allocation element is an edge with no W930."
verifies:
  - REQ-TRS-ALLOC-001
---

```gherkin
Feature: features-form allocations are carried only by Allocation elements (TC-TRS-ALLOC-003)

  Scenario: a features-form allocation on a PartDef raises W930
    Given a PartDef whose features: entry declares type: Allocation, allocatedFrom and allocatedTo
    When the tool validates the model
    Then W930 names the entry and the PartDef's file
    And matrix --allocations shows no edge for it

  Scenario: the same entry on an Allocation element is an edge
    Given a type: Allocation element carrying the same features: entry
    When the tool validates the model and prints matrix --allocations
    Then no W930 is reported and the edge ComputeThrust -> Board is shown
```
