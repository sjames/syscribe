---
id: TC-TRS-SYSMLV2-010
type: TestCase
testLevel: L3
status: draft
name: "Verify a named SysML v2 connection usage's connect endpoints lift onto the owning part's connections: field and resolve to real connectivity edges, both binary and n-ary form, with no regression for connect-less usages."
verifies:
  - REQ-TRS-SYSMLV2-010
---

```gherkin
Feature: connect endpoints lift onto the owning part's connections: field
  Scenario: a binary connect lifts and resolves to a real edge
    Given a SysMLv2 part def with two subparts and a named connection usage connecting them
    When connectivity is queried from one subpart
    Then a real edge to the other subpart is reported

  Scenario: an n-ary connect lifts to the ends: shape and every end resolves
    Given a SysMLv2 part def with three subparts and a named connection usage connecting all three
    When connectivity is queried from the first subpart
    Then a real edge to each of the other two subparts is reported

  Scenario: a connection usage with no connect clause contributes no entry
    Given a SysMLv2 part def with a named connection usage that has no connect clause
    When the model is exported
    Then the owning part carries no connections: field
```
