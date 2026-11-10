---
type: TestCase
id: TC-TRS-AGENTPOL-005
name: "A protected sidecar is written only by the commands sidecar_writers names"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-005
tags:
  - cli
  - agents
---

```gherkin
Feature: protecting recorded evidence without blocking ingest

  Scenario: sidecar protected, no writer named
    Then ingest-results by an agent exits 1, names protected_paths and the sidecar file, and writes nothing

  Scenario: ingest-results named in sidecar_writers
    Then the same ingest succeeds, policy says so in words, and a different named command does not open the path

  Scenario: the audit and the sidecar
    Then an agent's ingest-results --commit raises no W760, while another protected path in the same commit, and anything under .syscribe/scripts, still does

  Scenario: malformed sidecar_writers
    Then it grants nothing
```
