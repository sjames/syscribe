---
type: TestCase
id: TC-TRS-MCP-033
name: "why_active explains an element's activation under a configuration"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_variability.rs
verifies:
  - REQ-TRS-MCP-032
tags:
  - mcp
  - variability
---

```gherkin
Feature: Activation explanation

  Scenario: a gated element is inactive when its feature is deselected
    Given an initialized mcp server over a fixture with a feature model
    When why_active is called for REQ-FXSAT-001 under CONF-FX-001
    Then active is false
    And the effective appliesWhen references Features::Link::Sat

  Scenario: an ungated element is always active
    When why_active is called for REQ-FX-001 under CONF-FX-001
    Then active is true
```
