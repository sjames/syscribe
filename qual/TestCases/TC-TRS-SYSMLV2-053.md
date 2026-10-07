---
id: TC-TRS-SYSMLV2-053
type: TestCase
testLevel: L3
status: active
name: "Verify The parser dependency is evaluated for upgrade, its version is reported by syscribe sysml and sysml_submodels, and a drift guard keeps the reported version equal to the pin."
verifies:
  - REQ-TRS-SYSMLV2-053
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_include_values.rs
testFunctions:
  - reported_parser_version_equals_the_cargo_pin
  - report_json_carries_the_parser_block
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_include_values.rs`; run with `cargo test -p syscribe-model --test sysmlv2_include_values`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-053)

  Scenario: The parser dependency is evaluated for upgrade, its version is reported by syscribe sysml and sysml_submodels, and a drift guard keeps the reported version equal to the pin
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
