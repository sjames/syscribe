---
id: TC-TRS-BUDGET-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the budget expression language: a resolvable in-bound budget is clean; E866 (evaluate not a ConstraintDef), E867 (syntax error), E868 (unresolved operand), W060 (value violates the evaluate constraint); W060 draft-suppressed and gateable."
verifies:
  - REQ-TRS-BUDGET-001
---

Verify `bodyLanguage: budget` evaluation on `CalculationDef`.

```gherkin
Feature: Budget expression language (E866–E868, W060)

  Scenario: well-formed, in-bound budget is clean
    Given a CalculationDef budget whose operands resolve and whose value is within the evaluate constraint
    When the tool validates the model
    Then none of E866, E867, E868, W060 are emitted

  Scenario: E866 — evaluate target is not a ConstraintDef
    Given a budget CalculationDef whose evaluate points at a PartDef
    When the tool validates the model
    Then an E866 finding is emitted

  Scenario: E867 — malformed budget expression
    Given a budget body that does not parse against the grammar
    When the tool validates the model
    Then an E867 finding is emitted

  Scenario: E868 — unresolved operand
    Given a budget body referencing an attribute that resolves to no value
    When the tool validates the model
    Then an E868 finding is emitted

  Scenario: W060 — budget value violates the evaluate constraint
    Given a budget whose value exceeds the evaluate ConstraintDef bound
    When the tool validates the model
    Then a W060 finding is emitted

  Scenario: W060 draft-suppressed
    Given the violating budget with the CalculationDef status draft
    When the tool validates the model
    Then no W060 finding is emitted

  Scenario: --deny W060 promotes to a gate failure
    Given the violating budget
    When the tool validates the model with --deny W060
    Then the tool exits non-zero
```
