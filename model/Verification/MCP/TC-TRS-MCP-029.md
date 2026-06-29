---
type: TestCase
id: TC-TRS-MCP-029
name: "features lists the feature model and feature_check validates it"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_variability.rs
verifies:
  - REQ-TRS-MCP-028
tags:
  - mcp
  - variability
---

```gherkin
Feature: Feature-model inspection

  Scenario: features lists the feature definitions
    Given an initialized mcp server over a fixture with a feature model
    When the features tool is called with no argument
    Then hasFeatureModel is true
    And the feature list includes Features::Link::LoRa

  Scenario: feature_check validates the model
    When feature_check is called with deep=true
    Then a findings array is returned
    And the deep report reports the model is not void
```
