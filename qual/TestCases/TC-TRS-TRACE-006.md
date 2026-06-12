---
id: TC-TRS-TRACE-006
type: TestCase
testLevel: L3
status: draft
name: "Verify that E313 is emitted for incompatible domain/reqDomain in satisfies: links."
verifies:
  - REQ-TRS-TRACE-006
---

Verify that E313 is emitted for incompatible domain/reqDomain in satisfies: links.

```gherkin
Feature: Domain compatibility in satisfies: links

  Scenario: Software element satisfying hardware requirement produces E313
    Given a PartDef with domain: software
    And a Requirement with reqDomain: hardware
    And the PartDef has satisfies: pointing to that Requirement
    When the tool is invoked
    Then an E313 finding is emitted

  Scenario: Software element satisfying system requirement does not produce E313
    Given a PartDef with domain: software
    And a Requirement with reqDomain: system
    And the PartDef has satisfies: pointing to that Requirement
    When the tool is invoked
    Then no E313 finding is emitted

  Scenario: Software element satisfying software requirement does not produce E313
    Given a PartDef with domain: software
    And a Requirement with reqDomain: software
    And the PartDef has satisfies: pointing to that Requirement
    When the tool is invoked
    Then no E313 finding is emitted
```
