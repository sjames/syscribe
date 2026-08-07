---
id: TC-TRS-HPLE-002
type: TestCase
testLevel: L3
status: draft
name: "Verify parameterBindings reaches a parameter belonging to a peer FeatureDef reachable through subConfigurations, using its ordinary qname."
verifies:
  - REQ-TRS-HPLE-002
---

```gherkin
Feature: Transitive parameterBindings resolution through subConfigurations
  Scenario: a dotted key resolves a peer feature's parameter reachable through subConfigurations
    Given a Configuration with subConfigurations naming a peer Configuration
    And that Configuration's parameterBindings binds the peer FeatureDef's parameter by its ordinary qname
    When the model is validated
    Then no unresolved-reference error is raised for that binding

  Scenario: a dotted key naming a FeatureDef unreachable by any means is still rejected
    Given a Configuration whose parameterBindings binds a dotted key naming a FeatureDef that exists nowhere reachable
    When the model is validated
    Then an unresolved-reference error is raised
```
