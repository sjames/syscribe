---
id: TC-TRS-PIGATE-001
type: TestCase
testLevel: L3
status: active
name: "Verify a PlanningItem reference to an element gated out of the Configuration is not dangling, and a reference that exists nowhere still is."
verifies:
  - REQ-TRS-PIGATE-001
sourceFile: repo:crates/syscribe/tests/planning_gated_refs.rs
testFunctions:
  - a_reference_to_an_element_gated_out_of_the_configuration_is_not_dangling
  - a_parent_gated_out_leaves_the_step_valid
  - a_reference_that_exists_nowhere_is_still_an_error
---

```gherkin
Feature: PlanningItem references under --config (TC-TRS-PIGATE-001)

  Scenario: A reference to an element gated out of the Configuration is not dangling
    Given a planning item that achieves requirements gated on Layout and on Ab, and a Configuration selecting Layout only
    When validate --config and validate --all-configs run
    Then there is no E714 or E710 and the exit code is 0

  Scenario: A reference that exists nowhere is still an error
    Given a planning item achieving, blocked by and parented to ids that exist nowhere
    When validate runs, with and without --config
    Then E714, E720 and E710 are reported
```
