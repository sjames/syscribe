---
id: TC-TRS-SYSMLV2-088
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml writes metadata: entries as metadata annotations that read back identically."
verifies:
  - REQ-TRS-SYSMLV2-088
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - metadata_exports_as_annotations_and_reads_back
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-088)

  Scenario: export-sysml writes metadata: entries as metadata annotations that read back identically
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
