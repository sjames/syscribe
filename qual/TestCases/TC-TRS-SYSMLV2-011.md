---
id: TC-TRS-SYSMLV2-011
type: TestCase
testLevel: L3
status: draft
name: "Verify n2's scoped subpart axis includes SysMLv2-synthesized direct children and a lifted connection populates the off-diagonal cell, with no regression to unscoped n2 or the existing features:-only native n2 behavior."
verifies:
  - REQ-TRS-SYSMLV2-011
---

```gherkin
Feature: n2's scoped axis includes SysMLv2-synthesized children
  Scenario: scoped n2 on a SysMLv2 subtree lists its direct-child parts
    Given a SysMLv2 part def with two subparts wired by a named connection usage
    When n2 is run scoped to that part def
    Then both subparts appear on the diagonal

  Scenario: a lifted connection populates the off-diagonal cell
    Given the same scoped n2 run
    Then the off-diagonal cell between the two subparts names the connection

  Scenario: unscoped n2 is unaffected
    Given the same model
    When n2 is run with no scope
    Then it still lists every PartDef/Part exactly as before
```
