---
type: TestCase
id: TC-TRS-MCP-042
name: "diagram_coverage reports uncovered elements and unresolved shape refs"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_diagrams.rs
verifies:
  - REQ-TRS-MCP-041
tags:
  - mcp
---

```gherkin
Feature: Diagram coverage

  Scenario: an element in no diagram is reported as uncovered
    Given a fixture where REQ-FX-001 is referenced by no Diagram shape
    When diagram_coverage is called
    Then REQ-FX-001 appears in the uncovered set

  Scenario: an element referenced by a diagram shape is not uncovered
    Given Parts::Base is referenced by the FxBlock diagram
    When diagram_coverage is called
    Then Parts::Base does not appear in the uncovered set
```
