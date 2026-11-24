---
id: TC-TRS-VIEWTAB-003
type: TestCase
testLevel: L3
status: active
name: "Verify validate reports a malformed table view and an unknown field."
verifies:
  - REQ-TRS-VIEWTAB-003
sourceFile: repo:crates/syscribe/tests/view_table.rs
testFunctions:
  - a_well_formed_table_view_validates_without_w047_e507_or_w504
  - a_column_naming_an_unknown_field_is_w504
  - a_malformed_declaration_is_e507_and_render_refuses_it
---

```gherkin
Feature: Table view validation (TC-TRS-VIEWTAB-003)

  Scenario: A column names an unknown field
    Given a table view whose column names a field no exposed element has
    When the model is validated
    Then W504 names the column and the field
```
