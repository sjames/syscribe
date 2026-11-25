---
id: TC-TRS-DOCGEN-003
type: TestCase
testLevel: L3
status: active
name: "Verify doc build renders a Document and --check reports one that is out of date."
verifies:
  - REQ-TRS-DOCGEN-003
sourceFile: repo:crates/syscribe/tests/doc_template.rs
testFunctions:
  - doc_build_renders_the_document_and_check_passes_until_the_model_changes
  - check_reports_a_changed_template_a_hand_edit_a_missing_build_and_a_renamed_element
  - a_document_for_a_configuration_builds_that_variant
---

```gherkin
Feature: Building and checking documents (TC-TRS-DOCGEN-003)

  Scenario: A changed value makes the document out of date
    Given a Document that was built
    When a value its template uses changes in the model
    Then doc build --check exits 2 and shows the value as it is now
```
