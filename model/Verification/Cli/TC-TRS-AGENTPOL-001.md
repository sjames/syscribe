---
type: TestCase
id: TC-TRS-AGENTPOL-001
name: "Agent changes are refused where the policy forbids them"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-001
tags:
  - cli
---

```gherkin
Feature: Enforcing the agent policy

  Scenario: protected paths
    Then an agent's write is refused and names the rule, a person's is not, and a move that rewrites a protected file is refused

  Scenario: human-only statuses
    Then an agent cannot set one, a person can, and the refusal also happens under --dry-run

  Scenario: require_before_done and require_by
    Then an item without evidence cannot be marked done by an agent and a writer with no actor is refused

  Scenario: the write-layer backstop
    Then a command with no check of its own still cannot write a protected path
```
