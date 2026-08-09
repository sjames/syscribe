---
id: TC-TRS-SYSMLV2-009
type: TestCase
testLevel: L3
status: draft
name: "Verify SysML v2 doc /* ... */ comments lift into the synthesized element's doc body across part def/port def/interface usage, concatenate across multiple blocks, clear W600, and a no-doc element is unaffected."
verifies:
  - REQ-TRS-SYSMLV2-009
---

```gherkin
Feature: doc /* ... */ comments lift into the synthesized element's doc body
  Scenario: a single doc block lifts onto a part def and clears W600
    Given a SysMLv2 part def carrying one doc /* ... */ member
    When the model is validated
    Then the synthesized element's doc body matches the comment text and W600 does not fire for it

  Scenario: two doc blocks concatenate in source order
    Given a SysMLv2 part def carrying two doc /* ... */ members
    When the model is exported
    Then the synthesized element's doc body is both texts joined by a blank line, in source order

  Scenario: a part def with no doc member is unaffected
    Given a SysMLv2 part def with no doc /* ... */ member
    When the model is validated
    Then the synthesized element's doc body is empty and W600 still fires

  Scenario: the lift also reaches element kinds beyond part def
    Given a SysMLv2 port def and an interface usage, each carrying one doc /* ... */ member
    When the model is exported
    Then each synthesized element's doc body matches its own comment text
```
