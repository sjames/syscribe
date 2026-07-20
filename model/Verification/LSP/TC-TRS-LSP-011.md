---
type: TestCase
id: TC-TRS-LSP-011
name: "rename computes a WorkspaceEdit across every referencing file"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_rename.rs
verifies:
  - REQ-TRS-LSP-011
tags:
  - lsp
  - rename
---

```gherkin
Feature: rename

  Scenario: renaming a verified Requirement's id edits both the requirement and its TestCase
    Given a fixture model where a TestCase's `verifies:` names a Requirement's stable id
    When textDocument/rename is requested at the Requirement's id with a valid new id
    Then the response is a WorkspaceEdit with a TextEdit in the Requirement's own file
    And a TextEdit in the verifying TestCase's file
    And no file is written to disk by the server itself

  Scenario: a malformed new id is refused
    Given a fixture Requirement's id
    When textDocument/rename is requested with a new name that does not match the
      Requirement id pattern
    Then the response is an error, not a WorkspaceEdit
```
