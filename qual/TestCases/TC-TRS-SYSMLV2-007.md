---
id: TC-TRS-SYSMLV2-007
type: TestCase
testLevel: L3
status: draft
name: "Verify a file mixing mapped and unmapped SysMLv2 constructs parses fully and keeps only the mapped elements."
verifies:
  - REQ-TRS-SYSMLV2-007
---

```gherkin
Feature: parse-broad, map-narrow
  Scenario: a mapped construct survives alongside an unmapped one in the same file
    Given a single .sysml file containing both a mapped part def and an unmapped state def
    When the model is validated
    Then the part def appears as a first-class element under its derived qualified name

  Scenario: the unmapped construct contributes nothing
    Given the same file
    When the model's elements are listed
    Then no element or Finding is attributable to the unmapped construct
```
