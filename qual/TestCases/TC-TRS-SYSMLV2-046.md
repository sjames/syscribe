---
id: TC-TRS-SYSMLV2-046
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall lift a package-level SysMLv2 satisfy by-subject statement into the subject element satisfies: when the subject resolves."
verifies:
  - REQ-TRS-SYSMLV2-046
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - package_level_satisfy_lifts_into_the_subject_when_it_resolves
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-046)

  Scenario: Tool shall lift a package-level SysMLv2 satisfy by-subject statement into the subject element satisfies: when the subject resolves
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
