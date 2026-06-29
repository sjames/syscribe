---
type: TestCase
id: TC-TRS-MCP-044
name: "No MCP tool exposes a boolean property schema in its input schema"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-043
tags:
  - mcp
---

```gherkin
Feature: MCP tool input-schema validity

  Scenario: every tool property has an object schema
    Given an initialized mcp server
    When tools/list is requested
    Then for every tool, each entry under inputSchema.properties is a JSON object
    And no property's schema is a bare boolean (which strict zod clients reject)
```
