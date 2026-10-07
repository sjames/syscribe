---
id: TC-TRS-SYSMLV2-067
type: TestCase
testLevel: L3
status: active
name: "Verify equivalent spellings of a transition accept trigger read back as the same value."
verifies:
  - REQ-TRS-SYSMLV2-067
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export_close.rs
testFunctions:
  - accept_payload_mapping_exports_like_the_plain_string
  - accept_with_via_stays_a_mapping
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_export_close.rs`; run with `cargo test -p syscribe-model --test sysmlv2_export_close`.

```gherkin
Feature: SysMLv2 behaviour export closure (TC-TRS-SYSMLV2-067)

  Scenario: Equivalent spellings of a transition accept trigger read back as the same value
    Given a model using the feature
    When it is exported or ingested as the requirement states
    Then the observable result matches the requirement
```
