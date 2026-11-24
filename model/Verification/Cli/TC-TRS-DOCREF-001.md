---
id: TC-TRS-DOCREF-001
type: TestCase
testLevel: L3
status: active
name: "Verify render replaces the value references of an external document."
verifies:
  - REQ-TRS-DOCREF-001
sourceFile: repo:crates/syscribe/tests/doc_value_refs.rs
testFunctions:
  - a_tex_file_renders_with_values_units_and_latex_escaping
  - without_out_the_result_goes_to_stdout
  - a_configuration_gives_its_own_values
  - a_misspelt_reference_fails_with_file_and_line_and_writes_nothing
  - escaping_follows_the_file_type_and_the_flag_overrides_it
  - delimiters_come_from_the_flags_or_the_documents_table
  - a_malformed_documents_table_is_a_warning_and_the_entry_is_ignored
  - rendering_a_diagram_element_is_unchanged
---

```gherkin
Feature: Rendering an external document (TC-TRS-DOCREF-001)

  Scenario: A LaTeX file gets its values
    Given a .tex file with <<Limits.maxTemp>> and a [documents.delimiters] entry for tex
    When it is rendered
    Then the reference reads 120\,\textdegree{}C

  Scenario: A misspelt reference fails the render
    Given a .tex file with a reference to an attribute that does not exist
    When it is rendered with --out
    Then the command exits 1 naming the file and line, and writes nothing
```
