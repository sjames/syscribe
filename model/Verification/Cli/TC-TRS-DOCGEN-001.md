---
id: TC-TRS-DOCGEN-001
type: TestCase
testLevel: L3
status: active
name: "Verify render --template renders values, tables and element text, strictly."
verifies:
  - REQ-TRS-DOCGEN-001
sourceFile: repo:crates/syscribe/tests/doc_template.rs
testFunctions:
  - a_latex_template_gets_values_a_table_and_element_text
  - a_configuration_gives_its_values_and_selected_features
  - other_targets_use_jinja_delimiters_and_their_own_escaping
  - an_unknown_name_stops_the_render_with_the_template_line
  - out_writes_the_file_and_a_diagram_is_exported_beside_it
  - template_delimiters_are_configurable_and_includes_read_beside_the_template
---

```gherkin
Feature: Rendering a template (TC-TRS-DOCGEN-001)

  Scenario: A LaTeX template gets a value, a table and requirement text
    Given a template with value(), view() and a loop over elements()
    When it is rendered
    Then the value is set for LaTeX, the view is a longtable and each body is LaTeX

  Scenario: A renamed element fails the render
    Given a template naming an element that does not exist
    When it is rendered
    Then the command exits 1 with the template file and line, and writes nothing
```
