---
type: TestCase
id: TC-TRS-MCP-018
name: "Task-oriented authoring prompts are exposed"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-018
tags:
  - mcp
---

```gherkin
Feature: Task-oriented prompts

  Scenario: the convention prompts are listed
    Given an initialized mcp server
    When prompts/list is requested
    Then add-requirement, break-down-requirement, add-testcase-for, and traceability-review are present

  Scenario: a task prompt returns convention guidance
    When prompts/get is called for add-requirement
    Then it returns messages referencing derivedFrom and breakdownAdr conventions
```
