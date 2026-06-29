---
type: TestCase
id: TC-TRS-MCP-030
name: "configure reports completability of a configuration"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_variability.rs
verifies:
  - REQ-TRS-MCP-029
tags:
  - mcp
  - variability
---

```gherkin
Feature: Assisted configuration

  Scenario: a valid configuration is satisfiable
    Given an initialized mcp server over a fixture with a feature model
    When configure is called for CONF-FX-001
    Then the result reports satisfiable true
    And it reports the forced and free feature lists
```
