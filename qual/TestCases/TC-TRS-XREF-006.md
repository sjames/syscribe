---
id: TC-TRS-XREF-006
type: TestCase
testLevel: L3
status: draft
name: "Verify root-package-name hint: unresolved ref prefixed with the root name (stripped form resolves) gets a hint; correct ref no finding; non-matching ref no hint."
verifies:
  - REQ-TRS-XREF-006
---

```gherkin
Feature: hint when a cross-reference wrongly includes the model-root package name
  Scenario: a root-prefixed reference whose stripped form resolves gets a hint
    Given a derivedFrom reference written as <RootName>::A::B where A::B is a real requirement
    When the model is validated
    Then the usual unresolved-reference error is raised and it carries a hint naming A::B

  Scenario: the correctly written reference resolves with no finding
    Given the same reference written as A::B
    When the model is validated
    Then no unresolved-reference finding and no hint are produced

  Scenario: an unresolved reference not starting with the root name gets no hint
    Given a derivedFrom reference to a wholly unknown qualified name
    When the model is validated
    Then the unresolved-reference error is raised with no root-name hint
```
