---
id: TC-TRS-FMA-005
type: TestCase
testLevel: L3
status: draft
name: "Verify sound explanations for void models."
verifies:
  - REQ-TRS-FMA-005
---

```gherkin
Feature: Explanations for unsatisfiability
  Scenario: void explanation names the conflicting constraints
    Given a model void due to A requires B and A excludes B
    When feature-check --deep runs
    Then the E223 explanation names A, B, the requires and the excludes
  Scenario: soundness
    Given the same model with the excludes removed
    Then the model is no longer void
```
