---
type: TestCase
id: TC-TRS-MCP-028
name: "Project .syscribe.toml is readable as the config resource"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_retrieval.rs
verifies:
  - REQ-TRS-MCP-027
tags:
  - mcp
  - retrieval
---

```gherkin
Feature: Config resource

  Scenario: the config resource is listed and readable
    Given an initialized mcp server over a fixture model that has a .syscribe.toml
    When resources/read is issued for syscribe://config
    Then the project configuration text is returned
    And it reflects the project's id-prefix configuration
```
