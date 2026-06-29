---
type: TestCase
id: TC-TRS-MCP-025
name: "Logging capability is advertised and setLevel is honoured"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_live.rs
verifies:
  - REQ-TRS-MCP-024
tags:
  - mcp
---

```gherkin
Feature: MCP logging capability

  Scenario: logging capability is advertised
    Given an initialized mcp server
    Then the initialize result declares the logging capability

  Scenario: setLevel is accepted
    When a logging/setLevel request is sent
    Then it returns a success result
```
