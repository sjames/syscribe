---
id: TC-TRS-DERIVE-007
type: TestCase
testLevel: L3
status: draft
name: "Verify derive: is a recognised field (no W047), evaluates in dependency order, reports a malformed block as E505, and reports cyclic derive dependencies as E504 while skipping only the cyclic fields."
verifies:
  - REQ-TRS-DERIVE-001
  - REQ-TRS-DERIVE-004
---

```gherkin
Feature: derive: recognition and cycle detection (TC-TRS-DERIVE-007)

  Scenario: a derive: block raises no W047
    Given PartDefs declaring derive: blocks
    When the tool validates the model
    Then no W047 names the derive field

  Scenario: cross-element derived fields evaluate in dependency order
    Given Sys::Assembly (walked first) reading elements["Sys::Zeta"].zTotal + 1, with zTotal = 2 * 21
    When the tool shows Sys::Assembly
    Then its derived total is 43

  Scenario: a malformed derive: block raises E505
    Given an element with derive: 5 and another with a numeric formula value
    When the tool validates the model
    Then E505 is reported for each

  Scenario: a self-referential formula raises E504
    Given Sys::SelfRef with fieldA: self.fieldA + 1
    When the tool validates the model
    Then E504 names SelfRef.fieldA
    And fieldA is not evaluated

  Scenario: mutually dependent elements raise E504 on both
    Given Sys::Xray.a reading Sys::Yankee.b, which reads Sys::Xray.a
    When the tool validates the model
    Then E504 is reported on both files naming the cycle

  Scenario: a valid chain and a dependent of a cycle raise no E504
    Given a forward-referencing chain first -> second -> third and a field defaulting a cyclic reference with ??
    When the tool validates and shows the model
    Then no E504 names them, the chain evaluates to 9/8/4 and the dependent evaluates to 7
```
