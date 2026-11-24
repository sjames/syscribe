---
id: TC-TRS-DOCREF-003
type: TestCase
testLevel: L3
status: active
name: "Verify refs lists the uses of a value in the project's external documents."
verifies:
  - REQ-TRS-DOCREF-003
sourceFile: repo:crates/syscribe/tests/doc_value_refs.rs
testFunctions:
  - refs_lists_the_documents_that_use_an_attribute
  - refs_lists_the_documents_that_use_a_feature_parameter
  - without_a_documents_table_refs_reads_as_before
---

```gherkin
Feature: Document uses in refs (TC-TRS-DOCREF-003)

  Scenario: A value used in a LaTeX file is listed
    Given [documents] paths naming a directory with a .tex file that uses Rig.settle
    When refs Rig.settle is run
    Then the file and line are listed under Uses in documents
```
