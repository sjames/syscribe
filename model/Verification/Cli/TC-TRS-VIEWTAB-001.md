---
id: TC-TRS-VIEWTAB-001
type: TestCase
testLevel: L3
status: active
name: "Verify a View's declared columns, rows, filter, order, nesting and marks."
verifies:
  - REQ-TRS-VIEWTAB-001
sourceFile: repo:crates/syscribe/tests/view_table.rs
testFunctions:
  - rows_from_a_list_are_filtered_sorted_and_shown_in_the_declared_columns
  - a_view_with_columns_prints_as_a_table_by_default
  - nest_puts_children_under_their_parent_and_marks_flag_rows
  - parameter_rows_carry_effective_values_per_configuration
  - an_expose_filter_compares_fields
  - element_fields_computed_fields_and_element_prefix_resolve
---

```gherkin
Feature: Declared table views (TC-TRS-VIEWTAB-001)

  Scenario: One row per open question
    Given a view with rows: questions, where: status == open and three columns
    When it is rendered
    Then each open question of the exposed items is one row, in the declared columns and order

  Scenario: Children are nested under their parent
    Given a view of planning items with nest: parent
    When it is rendered
    Then a child item follows its parent, indented
```
