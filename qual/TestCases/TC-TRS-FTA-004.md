---
id: TC-TRS-FTA-004
type: TestCase
testLevel: L3
status: draft
name: "Verify fault-tree structural validation codes E980-E984 and W980-W984."
verifies:
  - REQ-TRS-FTA-004
---

Verify gate cycle, arity, range, reachability and stray-node findings.

```gherkin
Feature: Fault-tree structural validation

  Scenario: Gate cycle
    Given gates FTG-1 and FTG-2 that list each other as inputs
    When the tool validates the model
    Then E980 is emitted

  Scenario: Gate arity
    Given a NOT gate with two inputs and an inhibit gate with one input
    When the tool validates the model
    Then E981 is emitted

  Scenario: Value ranges
    Given an event with failureRate -1e-9 and probability 1.5
    When the tool validates the model
    Then E982 and E983 are emitted

  Scenario: Unreachable and stray nodes
    Given an event no gate references and an event outside the FaultTree directory
    When the tool validates the model
    Then W980 and W981 are emitted

  Scenario: Well-formed tree
    Given a well-formed AND tree
    When the tool validates the model
    Then none of E980-E984 or W980-W984 is emitted
```
