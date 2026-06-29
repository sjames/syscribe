---
type: TestCase
id: TC-TRS-MCP-011
name: "Tool failures return a structured error and the server keeps serving"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-011
tags:
  - mcp
---

Verifies that an unsatisfiable tool call returns a structured error result rather than crashing
the server, and that the server still serves a subsequent valid request.

```gherkin
Feature: Structured tool-error handling

  Scenario: an unresolved reference returns a tool error, not a crash
    Given an initialized mcp server over a fixture model
    When get_element is called with a reference that resolves to no element
    Then the tool result is flagged as an error
    And the server process is still running

  Scenario: the server serves a valid request after an error
    Given a prior tool call returned an error
    When get_element is called for an element that exists
    Then a well-formed result is returned for the valid reference
```
