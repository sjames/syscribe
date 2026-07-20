---
type: TestCase
id: TC-TRS-LSP-010
name: "prepareRename accepts a stable-id position and rejects everything else"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_rename.rs
verifies:
  - REQ-TRS-LSP-010
tags:
  - lsp
  - rename
---

```gherkin
Feature: prepareRename

  Scenario: prepareRename on a Requirement's own id succeeds
    Given a fixture Requirement's `id:` field
    When textDocument/prepareRename is requested at the cursor position on that id
    Then the response's placeholder is the current id

  Scenario: prepareRename on a name-identified element's qname position is refused
    Given a fixture PartDef (name-identified, not stable-id-identified)
    When textDocument/prepareRename is requested at a position resolving to that element
    Then the response is an error, not a range
```
