---
id: TC-TRS-SYSMLV2-052
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml shall emit constraint and calc parameters and expression bodies."
verifies:
  - REQ-TRS-SYSMLV2-052
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - export_constraint_and_calc_bodies_round_trip
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-052)

  Scenario: export-sysml shall emit constraint and calc parameters and expression bodies
    Given a native model using the feature
    When it is exported and the text re-ingested
    Then the SysML v2 text carries the construct and the re-ingested elements carry equal fields
```
