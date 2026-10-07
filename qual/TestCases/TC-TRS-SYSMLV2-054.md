---
id: TC-TRS-SYSMLV2-054
type: TestCase
testLevel: L3
status: active
name: "Verify A use-case include of a name that resolves to a use case in the submodel maps to the includes field; an unresolved include is counted as unmapped."
verifies:
  - REQ-TRS-SYSMLV2-054
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_include_values.rs
testFunctions:
  - include_resolves_to_a_use_case_in_scope_and_is_stored_in_includes
  - include_on_a_use_case_usage_resolves_too
  - unresolved_include_is_dropped_and_counted_as_unmapped
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_include_values.rs`; run with `cargo test -p syscribe-model --test sysmlv2_include_values`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-054)

  Scenario: A use-case include of a name that resolves to a use case in the submodel maps to the includes field; an unresolved include is counted as unmapped
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
