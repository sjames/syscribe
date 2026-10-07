---
id: TC-TRS-SYSMLV2-043
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall lift a SysMLv2 alias declared in a named package onto that Package element as an aliases: entry."
verifies:
  - REQ-TRS-SYSMLV2-043
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - alias_in_named_package_lifts_onto_package_aliases
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-043)

  Scenario: Tool shall lift a SysMLv2 alias declared in a named package onto that Package element as an aliases: entry
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
