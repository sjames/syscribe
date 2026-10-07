---
id: TC-TRS-SYSMLV2-080
type: TestCase
testLevel: L3
status: active
name: "Verify views, viewpoints, renderings and other already-mapped kinds nested in a part usage become native elements."
verifies:
  - REQ-TRS-SYSMLV2-080
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - views_and_other_kinds_nested_in_a_part_usage_are_native_elements
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-080)

  Scenario: Views, viewpoints, renderings and other already-mapped kinds nested in a part usage become native elements
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
