---
id: TC-TRS-FM-002
type: TestCase
testLevel: L3
status: draft
name: "Verify feature-check structural rules E212, E219, E220, W011, W012."
verifies:
  - REQ-TRS-FM-002
---

Verify requires/excludes resolution and satisfaction and dead/always-on optional detection under `feature-check`.

```gherkin
Feature: Feature model structural integrity

  Scenario: structural violations emit E212, E219, E220, W011, W012
    Given a feature model with an unresolved requires, a violated requires and
      excludes in a configuration, a dead optional feature, and an always-on
      optional feature
    When the tool runs feature-check
    Then findings E212, E219, E220, W011, and W012 are emitted

  Scenario: a clean feature model emits none of them
    Given a feature model with resolvable, satisfied requires and no dead or
      always-on optional features
    When the tool runs feature-check
    Then none of E212, E219, E220, W011, W012 are emitted
```
