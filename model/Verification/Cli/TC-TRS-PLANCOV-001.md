---
id: TC-TRS-PLANCOV-001
type: TestCase
testLevel: L3
status: active
name: "Verify a TestPlan accepts and checks its required-coverage declaration."
verifies:
  - REQ-TRS-PLANCOV-001
sourceFile: repo:crates/syscribe/tests/plan_coverage.rs
testFunctions:
  - a_well_formed_declaration_is_accepted_without_w047
  - explicit_points_are_accepted
  - a_malformed_declaration_is_e607
  - points_and_mode_together_are_e607
  - a_point_that_names_nothing_declared_is_e608
  - on_any_other_type_the_keys_stay_unrecognised
---

```gherkin
Feature: Required coverage on a TestPlan (TC-TRS-PLANCOV-001)

  Scenario: A well-formed declaration is accepted
    Given a TestPlan with conditions, minUnits and mode product
    When the model is validated
    Then no E607, E608 or W047 is raised

  Scenario: A malformed declaration is E607
    Given a TestPlan whose minUnits is 0 or whose axis is not a list
    When the model is validated
    Then E607 is raised

  Scenario: A dangling point is E608
    Given a points entry naming a point conditions does not declare
    When the model is validated
    Then E608 is raised
```
