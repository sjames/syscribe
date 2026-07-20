---
type: TestCase
id: TC-TRS-LSP-005
name: "Hover shows a resolved summary for the qname/id under the cursor"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_navigation.rs
verifies:
  - REQ-TRS-LSP-005
tags:
  - lsp
  - navigation
---

Verifies that `textDocument/hover` returns a Markdown summary (type, id, qname, status) for a
resolvable id/qname under the cursor, and no content for a dangling reference.

```gherkin
Feature: Hover

  Scenario: hover over a resolvable reference shows the target's summary
    Given a fixture model where a TestCase's `verifies:` names a Requirement's stable id
    When textDocument/hover is requested at the position of that id
    Then the response's contents include the Requirement's type, id, and qname

  Scenario: hover over a dangling reference returns no content
    Given a reference that does not resolve to any element
    When textDocument/hover is requested at its position
    Then the response is null
```
