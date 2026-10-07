---
id: TC-TRS-SYSMLV2-051
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml shall emit satisfies on non-part elements as satisfy statements and verifies on requirements as verify statements."
verifies:
  - REQ-TRS-SYSMLV2-051
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - export_satisfy_and_verify_become_real_statements
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-051)

  Scenario: export-sysml shall emit satisfies on non-part elements as satisfy statements and verifies on requirements as verify statements
    Given a native model using the feature
    When it is exported and the text re-ingested
    Then the SysML v2 text carries the construct and the re-ingested elements carry equal fields
```
