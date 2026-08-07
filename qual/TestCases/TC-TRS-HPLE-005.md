---
id: TC-TRS-HPLE-005
type: TestCase
testLevel: L3
status: draft
name: "Verify a lower tier's bindTo: cannot be repurposed to reach a higher tier's parameters, in either direction, while still working correctly within one model."
verifies:
  - REQ-TRS-HPLE-005
---

```gherkin
Feature: Lower-tier isolation from a consolidating higher tier
  Scenario: bindTo resolves normally within its own model (positive control)
    Given a model whose own Configuration binds the same key its own FeatureDef's bindTo names
    When that model's feature model is checked
    Then a propagation-range finding is raised for the out-of-range value

  Scenario: a lower tier's bindTo target never becomes visible to a separate higher tier
    Given a lower-tier model whose FeatureDef's bindTo names a dotted path matching a real parameter in a separate higher-tier model
    And the higher-tier model binds that same key to an out-of-range value
    When the higher-tier model's feature model is checked on its own
    Then no propagation-range finding is raised

  Scenario: a higher tier's binding never leaks down into the lower tier's own validation
    Given the same two models as above
    When the lower-tier model's feature model is checked on its own
    Then no propagation-range finding is raised
```
