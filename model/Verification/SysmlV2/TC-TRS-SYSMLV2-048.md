---
id: TC-TRS-SYSMLV2-048
type: TestCase
testLevel: L3
status: active
name: "Verify tool shall ingest multiplicity, subsets and redefines on SysMLv2 part, attribute, port and item usages."
verifies:
  - REQ-TRS-SYSMLV2-048
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - usage_multiplicity_subsets_and_redefines_are_ingested
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-048)

  Scenario: Tool shall ingest multiplicity, subsets and redefines on SysMLv2 part, attribute, port and item usages
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element carries the lifted fields and the construct is not counted by W543
```
