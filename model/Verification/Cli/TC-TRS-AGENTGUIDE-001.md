---
type: TestCase
id: TC-TRS-AGENTGUIDE-001
name: "agent-guide is built from the model and edits only its region"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_guide.rs
verifies:
  - REQ-TRS-AGENTGUIDE-001
tags:
  - cli
---

```gherkin
Feature: agent-guide

  Scenario: built from the model
    Then ids, status vocabularies, gates per profile and the safe-mutation rules reflect the model and .syscribe.toml

  Scenario: write
    Then only the marked region changes, a missing file is created and a file without a region gets one appended

  Scenario: check
    Then it fails after an id prefix changes until the region is regenerated

  Scenario: refusal
    Then damaged markers leave the file untouched and exit 1
```
