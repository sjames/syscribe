---
type: TestCase
id: TC-TRS-LSP-004
name: "Find-references returns every element that cross-references the element under the cursor"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_navigation.rs
verifies:
  - REQ-TRS-LSP-004
tags:
  - lsp
  - navigation
---

Verifies that `textDocument/references` returns the `Location` of every element whose
frontmatter cross-references the element under the cursor.

```gherkin
Feature: Find references

  Scenario: references on a Requirement finds its verifying TestCase
    Given a fixture model where a TestCase's `verifies:` names a Requirement's stable id
    When textDocument/references is requested at the position of the Requirement's id in its
      own file, with includeDeclaration true
    Then the response includes a Location for the verifying TestCase's file
    And the response includes a Location for the Requirement's own file (the declaration)

  Scenario: an element with no incoming references returns an empty list
    Given an element that nothing in the fixture model references
    When textDocument/references is requested at its own definition
    Then the response is an empty list, not an error
```
