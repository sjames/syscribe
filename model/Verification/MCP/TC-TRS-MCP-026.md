---
type: TestCase
id: TC-TRS-MCP-026
name: "A committed write emits a resource list-changed notification"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_live.rs
verifies:
  - REQ-TRS-MCP-025
tags:
  - mcp
---

```gherkin
Feature: Model-change notifications

  Scenario: resources capability advertises listChanged and subscribe
    Given an initialized mcp server
    Then the initialize result declares resources.listChanged and resources.subscribe

  Scenario: a committed write notifies the client
    Given an initialized mcp server over a fixture model
    When update_element commits a change with dry_run=false
    Then the client receives a notifications/resources/list_changed notification
```
