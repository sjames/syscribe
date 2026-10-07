---
id: TC-TRS-SYSMLV2-055
type: TestCase
testLevel: L3
status: active
name: "Verify An attribute usage with a literal value maps the value, and a literal-with-unit its unit, onto the Attribute element."
verifies:
  - REQ-TRS-SYSMLV2-055
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_include_values.rs
testFunctions:
  - attribute_literal_values_and_units_map_onto_the_attribute
  - attribute_value_and_unit_survive_export_and_re_ingestion
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_include_values.rs`; run with `cargo test -p syscribe-model --test sysmlv2_include_values`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-055)

  Scenario: An attribute usage with a literal value maps the value, and a literal-with-unit its unit, onto the Attribute element
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
