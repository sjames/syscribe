---
id: TC-TRS-VAL-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that each model-time error rule is triggered by its cross-element condition."
verifies:
  - REQ-TRS-VAL-002
---

Verify that each model-time error rule is triggered by its cross-element condition.

```gherkin
Feature: Model-time error rule enforcement

  Scenario Outline: Each model-time error code is produced by its trigger condition
    Given a multi-file model that satisfies the trigger condition for <code>
    When the tool is invoked
    Then at least one <code> finding is emitted

    Examples:
      | code  | trigger condition                                                             |
      | E101  | two elements share the same id: value                                         |
      | E102  | verifies: contains a reference that cannot be resolved                        |
      | E103  | derivedFrom: contains a reference that cannot be resolved                     |
      | E104  | verifies: reference resolves to a non-Requirement element                     |
      | E105  | derivedFrom: reference resolves to a non-Requirement element                  |
      | E106  | testFunctions[].scenario does not match any Gherkin scenario title            |
      | E310  | Requirement has derivedFrom: entries but no breakdownAdr:                     |
      | E311  | breakdownAdr: cannot be resolved or resolves to a non-ADR element             |
      | E312  | a parent Requirement appears in a satisfies: list                             |
      | E313  | satisfies: link connects incompatible domain and reqDomain values             |
      | E314  | isDeploymentPackage Part has no Allocation to a hardware element              |
      | E315  | element with domain: software has supertype: to a domain: hardware element    |
```
