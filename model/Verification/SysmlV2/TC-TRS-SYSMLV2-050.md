---
id: TC-TRS-SYSMLV2-050
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml shall emit an attribute unit as a SysML literal-with-unit instead of a comment."
verifies:
  - REQ-TRS-SYSMLV2-050
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - export_attribute_unit_is_a_literal_with_unit
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-050)

  Scenario: export-sysml shall emit an attribute unit as a SysML literal-with-unit instead of a comment
    Given a native model using the feature
    When it is exported and the text re-ingested
    Then the SysML v2 text carries the construct and the re-ingested elements carry equal fields
```
