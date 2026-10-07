---
id: TC-TRS-SYSMLV2-045
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall ingest a package-level SysMLv2 metadata def as a native MetadataDef."
verifies:
  - REQ-TRS-SYSMLV2-045
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - metadata_def_becomes_a_metadata_def_element
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-045)

  Scenario: Tool shall ingest a package-level SysMLv2 metadata def as a native MetadataDef
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
