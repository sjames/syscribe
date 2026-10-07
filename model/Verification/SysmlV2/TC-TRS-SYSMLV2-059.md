---
id: TC-TRS-SYSMLV2-059
type: TestCase
testLevel: L3
status: active
name: "Verify syscribe sysml and sysml_submodels count unresolved package-level satisfy statements and unresolved includes as unmapped."
verifies:
  - REQ-TRS-SYSMLV2-059
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_include_values.rs
testFunctions:
  - report_counts_unresolved_package_level_satisfy_like_w543
  - report_marks_unparsable_files_not_parsed
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_include_values.rs`; run with `cargo test -p syscribe-model --test sysmlv2_include_values`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-059)

  Scenario: syscribe sysml and sysml_submodels count unresolved package-level satisfy statements and unresolved includes as unmapped
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
