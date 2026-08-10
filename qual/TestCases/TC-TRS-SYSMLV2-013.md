---
id: TC-TRS-SYSMLV2-013
type: TestCase
testLevel: L3
status: draft
name: "Verify a two-segment connect endpoint resolves to a redeclared nested feature when one exists, falls back to head-only otherwise, and a three-segment chain always falls back."
verifies:
  - REQ-TRS-SYSMLV2-013
---

```gherkin
Feature: dotted connect endpoint resolution to a redeclared nested feature
  Scenario: a redeclared feature resolves to the finer-grained edge
    Given a SysMLv2 connect clause whose head parts both redeclare the referenced feature
    When connectivity is queried from the resolved feature
    Then a real edge to the other feature is reported

  Scenario: an inherited-only feature falls back to head-only
    Given a SysMLv2 connect clause whose head parts do not redeclare the referenced feature
    When the model is exported
    Then the connections: entry names the head parts only

  Scenario: a three-segment chain always falls back to head-only
    Given a SysMLv2 connect clause with a three-segment endpoint chain
    When the model is exported
    Then the connections: entry names the head only, even though the first segment is redeclared
```
