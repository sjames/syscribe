---
type: TestCase
id: TC-TRS-BDDCFG-001
name: "diagram export --config projects the model; BDD blocks show ASIL and responsibility"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/diagram_export_config.rs
verifies:
  - REQ-TRS-BDDCFG-001
tags:
  - vis
---

```gherkin
Feature: configuration-aware derived diagrams

  Scenario: projection
    Then a gated block is absent under the configuration that deselects it and present under the one that selects it

  Scenario: errors
    Then an unknown configuration exits 1

  Scenario: integrity
    Then the ASIL banner and responsibility line appear on the block
```
