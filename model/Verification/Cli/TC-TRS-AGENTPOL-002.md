---
type: TestCase
id: TC-TRS-AGENTPOL-002
name: "validate finds violations that bypassed the writers"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-002
tags:
  - cli
---

```gherkin
Feature: Auditing git for policy violations

  Scenario: W760 and W761
    Then an agent-made commit that changed a protected path or set a human-only status is reported

  Scenario: not agents
    Then a human commit or an as-human commit raises nothing

  Scenario: W762
    Then a malformed entry is reported and the rest of the policy still applies
```
