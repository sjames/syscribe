---
type: TestCase
id: TC-TRS-PHOLD-003
name: "feature card shows parameter consumers and per-configuration values; links and refs show placeholder references"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/placeholders_discovery.rs
verifies:
  - REQ-TRS-PHOLD-003
tags:
  - variability
---

```gherkin
Feature: placeholder discovery

  Scenario: feature card
    Then the JSON lists parameterConsumers and the value per configuration, and the text names them

  Scenario: links and refs
    Then links shows a placeholder relationship and refs of the feature lists the consumer
```
