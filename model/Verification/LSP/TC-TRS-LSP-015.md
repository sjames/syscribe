---
type: TestCase
id: TC-TRS-LSP-015
name: "codeAction offers an accept-as-reviewed quick-fix for W090, executed server-side"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_codeaction.rs
verifies:
  - REQ-TRS-LSP-015
tags:
  - lsp
  - codeaction
---

```gherkin
Feature: W090 quick-fix

  Scenario: a stale suspect link gets an "Accept as reviewed" command action
    Given a fixture model with a suspect (stale) trace link
    When textDocument/codeAction is requested over the source element's frontmatter range
    Then the response includes a CodeAction titled "Accept as reviewed" with a command
      referencing syscribe.suspectAccept

  Scenario: executing syscribe.suspectAccept clears the W090 finding
    Given the "Accept as reviewed" action's command and arguments
    When workspace/executeCommand is invoked with that command and arguments
    Then a subsequent reload shows the link is no longer suspect
```
