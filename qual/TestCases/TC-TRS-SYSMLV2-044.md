---
id: TC-TRS-SYSMLV2-044
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall ingest a SysMLv2 library package or namespace as a Package, at the root and nested."
verifies:
  - REQ-TRS-SYSMLV2-044
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - library_package_and_namespace_become_packages
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-044)

  Scenario: Tool shall ingest a SysMLv2 library package or namespace as a Package, at the root and nested
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
