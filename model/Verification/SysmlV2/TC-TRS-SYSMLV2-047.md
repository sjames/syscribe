---
id: TC-TRS-SYSMLV2-047
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall lift doc on a SysMLv2 requirement def or requirement onto the element doc text."
verifies:
  - REQ-TRS-SYSMLV2-047
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - doc_on_requirement_def_and_requirement_lifts
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-047)

  Scenario: Tool shall lift doc on a SysMLv2 requirement def or requirement onto the element doc text
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
