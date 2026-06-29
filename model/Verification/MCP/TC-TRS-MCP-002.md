---
type: TestCase
id: TC-TRS-MCP-002
name: "Model loads once into a shared store and the reload tool refreshes it"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-002
tags:
  - mcp
---

Verifies that the model is loaded once at startup and that the `reload` tool re-walks the model
from disk, picking up an external edit.

```gherkin
Feature: Shared model store and reload

  Scenario: repeated reads see a consistent snapshot
    Given an initialized mcp server over a fixture model
    When get_element is called twice for the same element without any edit
    Then both calls return identical content

  Scenario: reload picks up an externally added element
    Given an initialized mcp server over a fixture model
    And a new element file is written to the fixture on disk after startup
    When the reload tool is invoked
    Then the reported element count increases by one
    And a subsequent get_element for the new element resolves it
```
