---
type: TestCase
id: TC-TRS-MCP-043
name: "generate_view synthesises clean diagram source from the model graph"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_diagrams.rs
verifies:
  - REQ-TRS-MCP-042
tags:
  - mcp
---

```gherkin
Feature: View generation

  Scenario: a traceability view is generated as Mermaid source
    Given an initialized mcp server over the fixture model
    When generate_view is called with kind traceability
    Then Mermaid source is returned referencing REQ-FX-001 and its verifying test case
    And the source carries %% ref: annotations

  Scenario: a containment view is generated
    When generate_view is called with kind containment
    Then Mermaid source describing the namespace tree is returned
```
