---
type: TestCase
id: TC-TRS-LSP-006
name: "Workspace symbol search finds elements by name, id, or qualified name"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_navigation.rs
verifies:
  - REQ-TRS-LSP-006
tags:
  - lsp
  - navigation
---

Verifies that `workspace/symbol` finds elements across the whole model, matched by name, id,
or qualified name.

```gherkin
Feature: Workspace symbol search

  Scenario: query matches by stable id
    Given a fixture model containing a Requirement with a known id
    When workspace/symbol is requested with that id as the query
    Then the result includes a symbol whose location is the Requirement's file

  Scenario: an empty or non-matching query does not error
    Given the fixture model
    When workspace/symbol is requested with a query that matches nothing
    Then the result is an empty list, not an error
```
