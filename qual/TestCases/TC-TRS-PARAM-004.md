---
id: TC-TRS-PARAM-004
type: TestCase
testLevel: L3
status: draft
name: "Verify FeatureDef parameter bindingTime rules E230, E229, W027, and W017 suppression."
verifies:
  - REQ-TRS-PARAM-004
---

Verify that a parameter's `bindingTime:` is validated for vocabulary and ordering, that binding a runtime parameter in a Configuration warns, and that a runtime parameter suppresses the required-but-unbound warning.

```gherkin
Feature: FeatureDef parameter binding time

  Scenario: an unknown bindingTime value produces E230
    Given a FeatureDef parameter with bindingTime that is not compile/load/runtime
    When the tool validates the model
    Then an E230 finding is emitted

  Scenario: valid bindingTime values produce no E230
    Given FeatureDef parameters with bindingTime compile, load, and runtime
    When the tool validates the model
    Then no E230 finding is emitted

  Scenario: a parameter bound earlier than its derivedFrom source produces E229
    Given a compile-time parameter derivedFrom a runtime sibling, both with bindingTime
    When the tool runs feature-check
    Then an E229 finding is emitted

  Scenario: a parameter bound no earlier than its source produces no E229
    Given a runtime parameter derivedFrom a compile-time sibling, both with bindingTime
    When the tool runs feature-check
    Then no E229 finding is emitted

  Scenario: binding a runtime parameter in a Configuration produces W027
    Given a Configuration that binds a parameter declared bindingTime runtime
    When the tool validates the model
    Then a W027 finding is emitted

  Scenario: a required unbound runtime parameter suppresses W017
    Given a selected feature with a required runtime parameter left unbound
    When the tool validates the model
    Then no W017 finding is emitted

  Scenario: a required unbound non-runtime parameter still produces W017
    Given a selected feature with a required compile-time parameter left unbound
    When the tool validates the model
    Then a W017 finding is emitted
```
