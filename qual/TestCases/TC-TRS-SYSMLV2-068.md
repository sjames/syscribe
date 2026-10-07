---
id: TC-TRS-SYSMLV2-068
type: TestCase
testLevel: L3
status: active
name: "Verify expression spellings that ingestion canonicalises compare equal in the export read-back check."
verifies:
  - REQ-TRS-SYSMLV2-068
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export_close.rs
testFunctions:
  - equal_expression_spellings_export_as_text
  - a_genuinely_different_or_unparseable_expression_still_degrades
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_export_close.rs`; run with `cargo test -p syscribe-model --test sysmlv2_export_close`.

```gherkin
Feature: SysMLv2 behaviour export closure (TC-TRS-SYSMLV2-068)

  Scenario: Expression spellings that ingestion canonicalises compare equal in the export read-back check
    Given a model using the feature
    When it is exported or ingested as the requirement states
    Then the observable result matches the requirement
```
