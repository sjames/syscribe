---
type: TestCase
id: TC-TRS-MCPPOLICY-001
name: "MCP writes are held to the policy"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-001
  - REQ-TRS-AGENTPOL-003
tags:
  - cli
---

```gherkin
Feature: MCP and the agent policy

  Scenario: every write tool
    Then create, update, delete, move, question and apply_changes refuse a protected path or a human-only status

  Scenario: instructions
    Then the initialize instructions state the policy
```
