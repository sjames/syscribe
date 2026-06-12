---
id: TC-TRS-FTA-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that FaultTree, FaultTreeGate, and FaultTreeEvent validation rules E900–E909, W900–W901 are enforced."
verifies:
  - REQ-TRS-FTA-001
---

Verify that the tool detects and reports every validation error and warning defined for the `FaultTree`, `FaultTreeGate`, and `FaultTreeEvent` element types.

```gherkin
Feature: FaultTree, FaultTreeGate, and FaultTreeEvent validation rule enforcement

  Scenario Outline: Each FTA validation code is produced by its trigger condition
    Given a model fixture that satisfies the trigger condition for <code>
    When the tool validates the model
    Then at least one <code> finding is emitted

    Examples:
      | code  | trigger condition                                                                         |
      | E900  | FaultTree element is missing one or more of id, title, status, topEvent                  |
      | E901  | FaultTree id is present but does not match the FT-* pattern                               |
      | E902  | FaultTree topEvent resolves to an element that is not a SafetyGoal                        |
      | E903  | FaultTreeGate element is missing one or more of id, title, gateType                       |
      | E904  | FaultTreeGate id is present but does not match the FTG-* pattern                          |
      | E905  | FaultTreeGate gateType is not one of AND, OR, XOR, NOT, inhibit                           |
      | E906  | FaultTreeGate inputs entry does not resolve to a FaultTreeGate or FaultTreeEvent          |
      | E907  | FaultTreeEvent element is missing one or more of id, title, eventKind                     |
      | E908  | FaultTreeEvent id is present but does not match the FTE-* pattern                         |
      | E909  | FaultTreeEvent eventKind is not one of basic, undeveloped, house                          |
      | W900  | FaultTree has no FaultTreeGate or FaultTreeEvent children in its directory                 |
      | W901  | FaultTreeGate has no inputs field                                                          |
```
