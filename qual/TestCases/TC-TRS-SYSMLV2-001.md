---
id: TC-TRS-SYSMLV2-001
type: TestCase
testLevel: L3
status: draft
name: "Verify sysmlSubmodel: true scopes a subtree out of native parsing: subtree excluded, stray nested _index.md warned, .md siblings still parse, no-marker baseline unaffected."
verifies:
  - REQ-TRS-SYSMLV2-001
---

```gherkin
Feature: sysmlSubmodel: true scopes a package's subtree out of native Markdown parsing
  Scenario: the package's own _index.md still parses as a normal element
    Given a package _index.md declaring sysmlSubmodel: true
    When the model is validated
    Then the package itself appears as a normal Package element with no error

  Scenario: a stray nested _index.md is excluded and warned
    Given a _index.md nested inside the marked subtree, other than the package's own anchor
    When the model is validated
    Then a warning names the stray file and it is not processed as a package

  Scenario: a hand-authored .md sibling still parses normally
    Given a hand-authored .md element file alongside .sysml content in the marked subtree
    When the model is validated
    Then that element resolves under the package's qualified name with no error

  Scenario: a model with no sysmlSubmodel package is unaffected
    Given a model with no sysmlSubmodel: true declared anywhere
    When the model is validated
    Then validation produces zero errors and zero warnings related to this feature
```
