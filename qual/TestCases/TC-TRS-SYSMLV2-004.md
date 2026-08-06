---
id: TC-TRS-SYSMLV2-004
type: TestCase
testLevel: L3
status: draft
name: "Verify a native TestCase's verifies: field resolves against a SysMLv2-mapped element by qname, and still works unchanged against a native Requirement."
verifies:
  - REQ-TRS-SYSMLV2-004
---

```gherkin
Feature: native TestCase.verifies: targets a SysMLv2-mapped element
  Scenario: verifies: resolves against a SysMLv2-mapped element by qname
    Given a native TestCase whose verifies: names a SysMLv2-mapped element's qualified name
    When the model is validated
    Then no dangling-reference or wrong-type finding is raised for that entry

  Scenario: verifying a native Requirement still works unchanged
    Given a native TestCase whose verifies: names a real native Requirement
    When the model is validated
    Then no dangling-reference or wrong-type finding is raised for that entry

  Scenario: verifying a hand-authored non-Requirement element is still rejected
    Given a native TestCase whose verifies: names a hand-authored native PartDef
    When the model is validated
    Then the existing wrong-type finding is raised for that entry
```
