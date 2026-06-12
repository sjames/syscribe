---
id: TC-TRS-TRACE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that computed reverse indices are populated from downstream link fields."
verifies:
  - REQ-TRS-TRACE-001
---

Verify that computed reverse indices are populated from downstream link fields.

```gherkin
Feature: OSLC link direction and reverse indices

  Scenario: verifiedBy is computed from TestCase.verifies:
    Given a Requirement REQ-A-001
    And a TestCase with verifies: [REQ-A-001]
    When the tool builds the element graph
    Then REQ-A-001's computed verifiedBy list includes the TestCase's id

  Scenario: derivedChildren is computed from child Requirement.derivedFrom:
    Given a parent Requirement REQ-PARENT-001
    And two child Requirements each with derivedFrom: [REQ-PARENT-001]
    When the tool builds the element graph
    Then REQ-PARENT-001's derivedChildren count is 2
```
