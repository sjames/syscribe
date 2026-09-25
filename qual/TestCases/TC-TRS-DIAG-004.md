---
id: TC-TRS-DIAG-004
type: TestCase
testLevel: L3
status: draft
name: "Verify diagram measure, layout and compose fail with a not-found error for an unresolvable element."
verifies:
  - REQ-TRS-DIAG-004
---

```gherkin
Feature: diagram commands reject unknown elements (TC-TRS-DIAG-004)

  Scenario: measure an unknown element
    When the tool runs diagram measure Nope::X
    Then it exits 1, prints error: element 'Nope::X' not found on stderr and nothing on stdout

  Scenario: measure a mix of known and unknown elements
    When the tool runs diagram measure Arch::Pump,Nope::X
    Then it exits 1 and names only Nope::X

  Scenario: measure a known element
    When the tool runs diagram measure Arch::Pump
    Then it exits 0 and the JSON names Arch::Pump

  Scenario: layout and compose files naming an unknown element
    Given a placement file and a layout file each placing Arch::Pump and Nope::X
    When the tool runs diagram layout and diagram compose on them
    Then each exits 1 and names Nope::X
```
