---
type: TestCase
id: TC-TRS-MCP-031
name: "project returns a variant's active elements and validation"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_variability.rs
verifies:
  - REQ-TRS-MCP-030
tags:
  - mcp
  - variability
---

```gherkin
Feature: Variant projection

  Scenario: projecting a configuration yields its active elements
    Given an initialized mcp server over a fixture with a feature model
    When project is called for CONF-FX-001
    Then the resolved selection has Features::Link::LoRa selected
    And the active element set is non-empty
    And a findings array for the projected variant is returned
    And the satellite-only requirement REQ-FXSAT-001 is not active
```
