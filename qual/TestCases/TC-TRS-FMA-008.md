---
id: TC-TRS-FMA-008
type: TestCase
testLevel: L3
status: draft
name: "Verify the configure command: satisfiability, forced and free features, contradictions."
verifies:
  - REQ-TRS-FMA-008
---

```gherkin
Feature: Assisted configuration
  Scenario: forced and free features from a partial selection
    Given a model where A requires B and a partial selection {A: true}
    When the tool runs configure
    Then it reports satisfiable, B forced-true, and C free
  Scenario: contradictory partial selection
    Given a partial selection {A: true, B: false} with A requires B
    When the tool runs configure
    Then it reports unsatisfiable with an explanation and exits 1
  Scenario: dormant with no feature model
    Given a model with no FeatureDef
    Then configure prints a notice and exits 0
```
