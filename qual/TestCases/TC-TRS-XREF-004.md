---
id: TC-TRS-XREF-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that circular supertype chains are detected and reported without crashing."
verifies:
  - REQ-TRS-XREF-004
---

Verify that circular supertype chains are detected and reported without crashing.

```gherkin
Feature: Circular reference detection

  Scenario: Two-element supertype cycle is detected
    Given element A with supertype: B
    And element B with supertype: A
    When the tool is invoked
    Then a cycle-detection error is emitted
    And the tool exits normally without an infinite loop or stack overflow

  Scenario: Three-element supertype cycle is detected
    Given elements A, B, C where A → B → C → A via supertype:
    When the tool is invoked
    Then a cycle-detection error is emitted identifying the cycle
    And the tool exits normally
```
