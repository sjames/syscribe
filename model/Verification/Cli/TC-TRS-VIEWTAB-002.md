---
id: TC-TRS-VIEWTAB-002
type: TestCase
testLevel: L3
status: active
name: "Verify view render writes a table view as LaTeX, Markdown, CSV and HTML."
verifies:
  - REQ-TRS-VIEWTAB-002
sourceFile: repo:crates/syscribe/tests/view_table.rs
testFunctions:
  - latex_is_a_longtable_with_caption_label_and_escaping
  - markdown_html_and_json_carry_the_same_table
  - a_view_without_columns_prints_default_columns_in_a_document_format
  - out_writes_a_file_and_an_unknown_format_is_a_usage_error
---

```gherkin
Feature: Document formats of a table view (TC-TRS-VIEWTAB-002)

  Scenario: LaTeX output is a longtable
    Given a table view with a caption
    When it is rendered with --format latex
    Then the output is a longtable with the caption, a label and escaped cells
```
