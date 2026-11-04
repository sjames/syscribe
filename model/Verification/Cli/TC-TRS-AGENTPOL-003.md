---
type: TestCase
id: TC-TRS-AGENTPOL-003
name: "The policy is shown to agents over CLI, guide and MCP"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-003
tags:
  - cli
---

```gherkin
Feature: Showing the policy

  Scenario: policy command and agent-guide
    Then both state the rules and say how this invocation is treated

  Scenario: no policy
    Then they say none is configured
```
