---
id: TC-TRS-SYSMLV2-071
type: TestCase
testLevel: L3
status: active
name: "Verify a repository regression test bounds the number of behavioural entries export-sysml cannot write."
verifies:
  - REQ-TRS-SYSMLV2-071
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export_ratchet.rs
testFunctions:
  - behaviour_export_degradations_do_not_exceed_the_budget
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_export_ratchet.rs`; run with `cargo test -p syscribe-model --test sysmlv2_export_ratchet`.

```gherkin
Feature: SysMLv2 behaviour export closure (TC-TRS-SYSMLV2-071)

  Scenario: A repository regression test bounds the number of behavioural entries export-sysml cannot write
    Given a model using the feature
    When it is exported or ingested as the requirement states
    Then the observable result matches the requirement
```
