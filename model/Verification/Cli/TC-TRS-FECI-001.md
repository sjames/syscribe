---
type: TestCase
id: TC-TRS-FECI-001
name: "The fixture matches its expectations and every published contract holds on it"
status: active
testLevel: L3
sourceFile: repo:crates/syscribe/tests/frontend_contract.rs
verifies:
  - REQ-TRS-FECI-001
tags:
  - lsp
---

```gherkin
Feature: front-end contract tests

  Scenario: the fixture
    Then it validates with no errors, contains what EXPECTED.json says and gives the documented answers

  Scenario: every transport
    Then every published CLI contract validates, every LSP request answers validly and announces a saved change, and the MCP server offers the advertised read tools and hides write tools when read-only

  Scenario: the handshake
    Then the capabilities a front end requires are present and absent ones read as absent
```
