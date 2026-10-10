---
type: TestCase
id: TC-TRS-CFGRES-001
name: "per-configuration ingest, overlay, matrix columns and run history"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_by_config.rs
verifies:
  - REQ-TRS-CFGRES-001
tags:
  - cli
---

```gherkin
Feature: per-configuration evidence

  Scenario: ingest
    Then --config stores the results under that configuration only, and an unknown configuration exits 1 without writing

  Scenario: matrix columns
    Given a test that fails on one configuration and passes on another
    Then each matrix column shows its own verdict

  Scenario: overlay
    Then a configuration without its own verdict falls back to the global one

  Scenario: history
    Then run records keep per-configuration sections and diff names the configuration
```
