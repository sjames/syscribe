---
type: TestCase
id: TC-TRS-LSP-016
name: "codeAction offers no quick-fix for diagnostics other than E310 and W090"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_codeaction.rs
verifies:
  - REQ-TRS-LSP-016
tags:
  - lsp
  - codeaction
---

```gherkin
Feature: codeAction scoping

  Scenario: a clean element with no E310/W090 condition gets no code actions
    Given a fixture Requirement with no missing breakdownAdr and no suspect links
    When textDocument/codeAction is requested over that Requirement's frontmatter range
    Then the response is an empty list, not an error
```
