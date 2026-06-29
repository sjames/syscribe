---
type: TestCase
id: TC-TRS-MCP-001
name: "mcp subcommand completes the MCP initialize handshake over stdio"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-001
tags:
  - mcp
---

Verifies that `syscribe mcp` starts a server that speaks the MCP protocol over stdio and
completes the `initialize` handshake, advertising the `tools`, `resources`, and `prompts`
capabilities.

```gherkin
Feature: MCP server startup and handshake

  Scenario: initialize handshake succeeds over stdio
    Given a fixture model directory
    When syscribe mcp -m <fixture> is spawned and an initialize request is sent on stdin
    Then a JSON-RPC initialize response is returned on stdout
    And the response advertises a protocol version accepted by the client
    And the response declares tools, resources, and prompts capabilities
    And no protocol bytes are written to stderr

  Scenario: tools are listed
    Given the server has completed the initialize handshake
    When a tools/list request is sent
    Then the result includes get_element, search, validate, and create_element among the tools

  Scenario: clean shutdown on stdin close
    Given the server has completed the initialize handshake
    When stdin is closed
    Then the process exits with code 0
```
