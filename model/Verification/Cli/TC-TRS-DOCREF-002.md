---
id: TC-TRS-DOCREF-002
type: TestCase
testLevel: L3
status: active
name: "Verify lint-docs checks .tex and other text documents."
verifies:
  - REQ-TRS-DOCREF-002
sourceFile: repo:crates/syscribe/tests/doc_value_refs.rs
testFunctions:
  - lint_docs_checks_a_tex_file
  - the_file_types_read_in_a_directory_are_selectable
  - a_clean_tex_file_passes_and_w098_can_be_denied
---

```gherkin
Feature: Linting other document types (TC-TRS-DOCREF-002)

  Scenario: A stale id in a LaTeX file is reported
    Given a .tex file naming a requirement id that does not exist
    When lint-docs scans its directory
    Then W099 names the file, the line and the id

  Scenario: A broken value reference is reported
    Given a .tex file with a reference to an attribute that does not exist
    When lint-docs scans it
    Then W098 is raised and the run fails
```
