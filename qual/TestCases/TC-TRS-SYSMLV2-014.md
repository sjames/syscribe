---
id: TC-TRS-SYSMLV2-014
type: TestCase
testLevel: L3
status: draft
name: "Verify a doc-comment @Syscribe*: directive lifts shortName/implementedBy onto an interface def and domain/asilLevel onto a port def, drives W023, strips the directive line from doc:, and leaves an unrecognized @...: line and a plain interface def unaffected."
verifies:
  - REQ-TRS-SYSMLV2-014
---

```gherkin
Feature: doc-comment @Syscribe* directives on interface def/port def/connection def
  Scenario: an interface def's doc-comment directives lift shortName and implementedBy
    Given an interface def whose doc comment contains @SyscribeShortName: and @SyscribeImplementedBy: lines
    When the model is exported
    Then shortName and implementedBy are set on the synthesized element
    And the directive lines do not appear in its doc text, while surrounding prose does

  Scenario: implementedBy lifted via a directive drives W023
    Given the lifted implementedBy path does not exist on disk
    When the model is validated
    Then W023 is raised for that element

  Scenario: a port def's doc-comment directives lift domain and asilLevel
    Given a port def whose doc comment contains @SyscribeDomain: and @SyscribeIntegrity: lines
    When the model is exported
    Then domain and asilLevel are set on the synthesized element

  Scenario: a connection def's doc-comment directive lifts shortName
    Given a connection def whose doc comment contains only an @SyscribeShortName: line
    When the model is exported
    Then shortName is set and the doc text is empty

  Scenario: an interface def with no doc comment is unaffected
    Given an interface def with no doc comment at all
    When the model is exported
    Then no Syscribe-lifted field is set on it
```
