---
type: TestCase
id: TC-TRS-MCPPROV-001
name: "MCP writes carry provenance, log reads it, read-only hides the writers"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_provenance.rs
verifies:
  - REQ-TRS-PROVLOG-001
  - REQ-TRS-WRITEGUARD-001
tags:
  - cli
---

```gherkin
Feature: MCP provenance

  Scenario: commit
    Then create, update, move, delete and question commit only their own files with the actor and reason

  Scenario: stale hash
    Then each refuses with nothing written

  Scenario: read-only
    Then exactly the tools not annotated read-only are hidden
```
