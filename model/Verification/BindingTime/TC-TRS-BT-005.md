---
id: TC-TRS-BT-005
type: TestCase
testLevel: L3
status: active
name: "Verify binding-times, its MCP tool and the agent instructions list the vocabulary"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-005.sh
verifies:
  - REQ-TRS-BT-005
---

```gherkin
Feature: Binding-time discovery
  Scenario: CLI and JSON
    Given a project declaring a deploy stage
    When binding-times --json is run twice
    Then the output is identical and ordered earliest to latest

  Scenario: agent instructions
    Given the same project
    When --agent-instructions is run
    Then a Project binding times section lists the declared stages
```
