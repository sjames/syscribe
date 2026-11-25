---
id: TC-TRS-DOCGEN-002
type: TestCase
testLevel: L3
status: active
name: "Verify the Document element validates, is listed and reports a malformed declaration."
verifies:
  - REQ-TRS-DOCGEN-002
sourceFile: repo:crates/syscribe/tests/doc_template.rs
testFunctions:
  - a_document_element_validates_and_is_listed
  - a_malformed_document_is_e508_and_a_missing_template_w505
---

```gherkin
Feature: The Document element (TC-TRS-DOCGEN-002)

  Scenario: A Document with no template is an error
    Given a Document element without template:
    When the model is validated
    Then E508 names the missing field
```
