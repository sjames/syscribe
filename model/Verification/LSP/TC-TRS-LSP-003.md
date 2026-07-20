---
type: TestCase
id: TC-TRS-LSP-003
name: "Go-to-definition resolves a qname/id cross-reference to its defining file"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_navigation.rs
verifies:
  - REQ-TRS-LSP-003
tags:
  - lsp
  - navigation
---

Verifies that `textDocument/definition` resolves the stable id or qualified name under the
cursor to the `Location` of its defining file.

```gherkin
Feature: Go to definition

  Scenario: definition on a verifies reference resolves to the target Requirement's file
    Given a fixture model where a TestCase's `verifies:` names a Requirement's stable id
    When textDocument/definition is requested at the position of that id in the TestCase file
    Then the response is a Location whose uri is the Requirement's defining file

  Scenario: definition on a non-reference position returns no location
    Given an open file
    When textDocument/definition is requested at a position with no resolvable id/qname
    Then the response is null
```
