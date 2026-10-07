---
id: TC-TRS-SYSMLV2-066
type: TestCase
testLevel: L3
status: active
name: "Verify ingested multiplicity with a reversed or non-natural bound raises advisory W544."
verifies:
  - REQ-TRS-SYSMLV2-066
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - reversed_or_non_natural_multiplicity_raises_w544
  - negative_or_fractional_multiplicity_bound_raises_w544
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-066)

  Scenario: Ingested multiplicity with a reversed or non-natural bound raises advisory W544
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
