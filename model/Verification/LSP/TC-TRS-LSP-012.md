---
type: TestCase
id: TC-TRS-LSP-012
name: "rename refuses a candidate that would introduce a new unresolved reference"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_rename.rs
verifies:
  - REQ-TRS-LSP-012
tags:
  - lsp
  - rename
---

```gherkin
Feature: rename safety gate

  Scenario: renaming to an id that collides with an existing element is refused
    Given two fixture Requirements with distinct stable ids
    When textDocument/rename is requested on the first, with the second's id as the new name
    Then the response is an error, not a WorkspaceEdit
```
