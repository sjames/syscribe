---
id: TC-TRS-SYSMLV2-065
type: TestCase
testLevel: L3
status: active
name: "Verify compound unit expressions ingest with or without spaces and export as unit expressions, not quoted names."
verifies:
  - REQ-TRS-SYSMLV2-065
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - compound_units_ingest_with_or_without_spaces_and_export_as_expressions
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-065)

  Scenario: Compound unit expressions ingest with or without spaces and export as unit expressions, not quoted names
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
