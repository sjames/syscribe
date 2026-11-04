---
type: TestCase
id: TC-TRS-CAPS-001
name: "capabilities describes the installed binary and matches the LSP handshake"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/json_contract.rs
verifies:
  - REQ-TRS-CAPS-001
tags:
  - cli
---

```gherkin
Feature: capabilities handshake

  Scenario: the document
    Then it names the version, commands with options, contracts, optional features and MCP tools

  Scenario: LSP parity
    Then the initialize result carries the same document
```
