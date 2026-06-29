---
type: TestCase
id: TC-TRS-MCP-032
name: "diff_configs reports elements differing between two variants"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_variability.rs
verifies:
  - REQ-TRS-MCP-031
tags:
  - mcp
  - variability
---

```gherkin
Feature: Variant diff

  Scenario: a satellite-gated element appears only in the satellite variant
    Given an initialized mcp server over a fixture with a feature model
    When diff_configs is called with a=CONF-FX-001 and b a selection of the satellite link
    Then REQ-FXSAT-001 is reported only in B
```
