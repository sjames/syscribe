---
type: TestCase
id: TC-TRS-LSP-014
name: "codeAction offers a breakdownAdr quick-fix for E310, one action per accepted ADR"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_codeaction.rs
verifies:
  - REQ-TRS-LSP-014
tags:
  - lsp
  - codeaction
---

```gherkin
Feature: E310 quick-fix

  Scenario: a Requirement missing breakdownAdr gets one action per accepted ADR
    Given a fixture Requirement with derivedFrom set and no breakdownAdr, and one accepted ADR
    When textDocument/codeAction is requested over that Requirement's frontmatter range
    Then the response includes exactly one CodeAction whose edit inserts
      `breakdownAdr: <the accepted ADR's qname>`
```
