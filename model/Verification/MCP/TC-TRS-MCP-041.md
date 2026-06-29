---
type: TestCase
id: TC-TRS-MCP-041
name: "render_diagram returns diagram source and structural findings, never an image"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_diagrams.rs
verifies:
  - REQ-TRS-MCP-040
tags:
  - mcp
---

```gherkin
Feature: Diagram source

  Scenario: a SysML diagram yields PlantUML source
    Given a fixture BDD Diagram element
    When render_diagram is called with format plantuml
    Then the result contains PlantUML source and a findings array
    And no rendered image artifact is returned

  Scenario: a Mermaid diagram yields its Mermaid source
    When render_diagram is called for the Mermaid Diagram
    Then the result contains the Mermaid source
```
