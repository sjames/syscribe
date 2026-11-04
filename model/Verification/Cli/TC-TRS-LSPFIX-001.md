---
type: TestCase
id: TC-TRS-LSPFIX-001
name: "Each product-line quick fix removes its diagnostic and is validated first"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_ple_actions.rs
verifies:
  - REQ-TRS-LSPFIX-001
tags:
  - lsp
---

```gherkin
Feature: product-line quick fixes

  Scenario: one test per code
    Then E222, E754, E751, E752, E756, W016 and E201 each get a fix whose edit makes the diagnostic disappear when applied

  Scenario: validated first
    Then a stage that would break the derivation chain is not offered for E751, and a file with none of these findings gets no fixes and is never written
```
