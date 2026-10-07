---
id: TC-TRS-SYSMLV2-036
type: TestCase
testLevel: L3
status: active
name: "Verify a package-level SysMLv2 doc comment lifts onto the synthesized Package element's doc text."
verifies:
  - REQ-TRS-SYSMLV2-036
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs
testFunctions:
  - package_level_doc_lifts_onto_the_package_and_is_no_longer_unmapped
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs`; run with `cargo test -p syscribe-model --test sysmlv2_constraints_calcs`.

```gherkin
Feature: pkgdoc mapping (TC-TRS-SYSMLV2-036)

  Scenario: ingested construct becomes a native element
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element exists with the lifted fields and the construct is not counted by W543
```
