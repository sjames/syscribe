---
id: TC-TRS-SEC-009
type: TestCase
testLevel: L3
status: draft
name: "Verify the attack-tree feasibility roll-up starts at the tree's root node, independent of file order."
verifies:
  - REQ-TRS-SEC-009
---

Verify that an `AttackTree`'s computed feasibility is rolled up from its root
node — the gate/step no other gate of the tree lists in `inputs:` — and not from
whichever gate/step sorts first in the directory (GH #149).

```gherkin
Feature: attack-tree roll-up root is structural, not file-order dependent

  Scenario: the roll-up starts at the gate no other gate lists as an input
    Given an AttackTree AT-ROOT-001 whose root OR gate ATG-ROOT-002 sorts after
      its AND sub-gate ATG-ROOT-001 (min(high, low) = low),
      the root being max(low, medium) = medium,
      and ThreatScenario TS-ROOT-001 declares attackFeasibility medium
    When the tool validates the model
    Then no W035 finding is emitted and there are no errors

  Scenario: a mismatch is still reported against the root's value
    Given the same tree but TS-ROOT-001 declares attackFeasibility high
    When the tool validates the model
    Then a W035 finding names computed medium and declared high
```
