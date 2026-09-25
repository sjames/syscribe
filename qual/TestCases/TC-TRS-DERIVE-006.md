---
id: TC-TRS-DERIVE-006
type: TestCase
name: "Derive codes E505/E506 and Allocation codes E500–E503 are disjoint: one model raising both families reports each under its own code"
status: approved
testLevel: L2
verifies: [REQ-TRS-DERIVE-005, REQ-TRS-VAL-009]
---

One model carries a derive parse error, a derive unknown-element reference, and all four
unresolved-Allocation forms. Every finding must carry the code of its own family — no code is
shared between the derive pass and Allocation resolution (GH #127).

```gherkin
Feature: derive and Allocation finding codes are disjoint

  Scenario: a model raising both families reports each under its own code
    Given a model with derive formulas "sum(" and 'elements["Gone::Thing"].x'
    And an inline Allocation feature and a Part with unresolved allocatedFrom/allocatedTo
    When the model is validated
    Then E505 names the derive parse error and E506 names 'Gone::Thing'
    And E500, E501, E502 and E503 each name only an Allocation reference
    And no derive message is reported under E500-E503
```
