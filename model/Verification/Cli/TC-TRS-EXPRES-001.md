---
id: TC-TRS-EXPRES-001
type: TestCase
testLevel: L3
status: active
name: "Verify export --resolved carries bodies, effective parameters, reverse links, questions and build identity."
verifies:
  - REQ-TRS-EXPRES-001
sourceFile: repo:crates/syscribe/tests/export_resolved.rs
testFunctions:
  - the_plain_export_is_unchanged
  - every_element_carries_its_body_raw_and_with_value_references_rendered
  - under_a_configuration_the_body_holds_that_configurations_values
  - a_configuration_carries_its_effective_parameters
  - with_config_the_top_level_holds_that_configurations_parameters
  - reverse_links_are_computed
  - a_planning_item_carries_its_questions_and_the_open_count
  - meta_names_the_build
  - ndjson_carries_meta_in_the_header_and_the_members_on_each_line
  - the_schema_is_documented_under_spec_and_help
---

```gherkin
Feature: Resolved export (TC-TRS-EXPRES-001)

  Scenario: A requirement's text is in the export
    Given a requirement whose body holds a value reference
    When the model is exported with --resolved
    Then the element carries the body as written and with the value rendered

  Scenario: A Configuration's parameters have their effective values
    Given a Configuration that binds one parameter and a second derived from it
    When the model is exported with --resolved
    Then the Configuration carries both values, with unit, range and the expression

  Scenario: The plain export does not change
    Given any model
    When it is exported without --resolved
    Then no element carries a body and there is no meta
```
