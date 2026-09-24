---
id: TC-TRS-LINKTYPE-012
type: TestCase
testLevel: L3
status: draft
name: "Verify an LLM agent can discover link types: the prompt documents links: and link-types, and --agent-instructions with a model appends the project's declared types."
verifies:
  - REQ-TRS-LINKTYPE-012
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-012)

  Scenario: the general prompt documents links: and link-types
  Scenario: a model declaring link types appends a Project link types section
  Scenario: a model declaring none appends nothing
```
