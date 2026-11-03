---
type: TestCase
id: TC-TRS-MCPGUIDE-001
name: "MCP instructions and workflow prompts match the model and the tools"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_instructions_prompts.rs
verifies:
  - REQ-TRS-MCPGUIDE-001
tags:
  - mcp
---

```gherkin
Feature: MCP guidance

  Scenario: initialize instructions
    Then they name the project's real id prefixes, statuses and ownership rule, and say READ-ONLY on a read-only server

  Scenario: print-instructions
    Then the CLI text equals the initialize response

  Scenario: workflow prompts
    Then all five are listed with their arguments, name only real tools, substitute arguments and refuse a missing required one

  Scenario: end to end
    Then the add-requirement-with-test sequence creates a requirement and a verifying test, and changes / validate new_since work against git
```
