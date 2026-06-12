---
id: TC-TRS-LINK-003
type: TestCase
testLevel: L3
status: draft
name: "Verify Mermaid diagrams emit click directives to the hosted URL."
verifies:
  - REQ-TRS-LINK-003
---

Verify that, with `[links]` configured, a rendered Mermaid diagram appends `click <nodeId> "<url>" _blank` directives to the hosted URL, and that with no `[links]` table no hosted `click` directive is emitted.

```gherkin
Feature: clickable Mermaid nodes

  Scenario: configured links append a click directive per node
    Given a model with [links] configured and a Mermaid diagram with %% link: directives
    When the diagram is rendered
    Then the output contains click <node> "<hosted url>" _blank for each linked node

  Scenario: no [links] table emits no hosted click directive
    Given a model with no [links] configured
    When the diagram is rendered
    Then the output contains no click directive to a hosted URL
```
