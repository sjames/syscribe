---
id: TC-TRS-AW-002
type: TestCase
testLevel: L3
status: draft
name: "Verify applies-when read mode: own gate, inherited (package) effective gate, always-applies, read-only, --json, unresolved."
verifies:
  - REQ-TRS-AW-002
---

```gherkin
Feature: applies-when read mode (display an element's gate)
  Scenario: own gate
    When applies-when is run on an element with its own appliesWhen
    Then its own expression and an equal effective condition are shown

  Scenario: inherited (package) gate
    When applies-when is run on an element gated only by an ancestor package
    Then it shows no own gate and an effective condition inherited from that package

  Scenario: ungated element
    When applies-when is run on an element with no own or ancestor gate
    Then it reports that the element always applies

  Scenario: read-only
    When applies-when is run in read mode
    Then the element file is not modified

  Scenario: json output
    When applies-when --json is run
    Then a structured object with effective and inheritedFrom is emitted

  Scenario: unresolved element
    When applies-when names an element that does not exist
    Then the command exits non-zero
```
