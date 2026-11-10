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

  Scenario: the audit and the sidecar, default exempt
    Then an agent's ingest-results --commit raises no W760, while another protected path in the same commit, and anything under .syscribe/scripts, still does

  Scenario: sidecar_audit command
    Then a tool commit by a listed writer is clean, a plain git commit or another command's commit of the sidecar is W760 with a hint naming --commit and sidecar_audit, another protected path is still reported, and a bad value is W762 falling back to exempt

  Scenario: an agent is told how to configure it
    Then a refused sidecar write names sidecar_writers (or the commands that may write it), a non-sidecar protected path gets no such advice, and policy states the audit mode

  Scenario: a moved runs_dir
    Then a listed writer may write a runs_dir inside the model, in the repo outside the model and outside the repo, an unlisted one is refused with the hint, the audit reports a plain commit of it under sidecar_audit command and not a tool commit, and a runs_dir that is .syscribe is a warning with the default applied

  Scenario: malformed sidecar_writers
    Then it grants nothing
```
