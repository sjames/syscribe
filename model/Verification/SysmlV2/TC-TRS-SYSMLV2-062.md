---
id: TC-TRS-SYSMLV2-062
type: TestCase
testLevel: L3
status: active
name: "Verify export never emits a succession whose endpoint entry was not exported."
verifies:
  - REQ-TRS-SYSMLV2-062
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - succession_with_a_commented_endpoint_is_commented_too
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-062)

  Scenario: Export never emits a succession whose endpoint entry was not exported
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
