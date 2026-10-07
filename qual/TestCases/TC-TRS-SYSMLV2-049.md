---
id: TC-TRS-SYSMLV2-049
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml shall emit subsets and redefines on usages and round-trip multiplicity."
verifies:
  - REQ-TRS-SYSMLV2-049
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_gaps.rs
testFunctions:
  - export_round_trips_multiplicity_subsets_and_redefines
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_gaps`.

```gherkin
Feature: SysMLv2 gap closure (TC-TRS-SYSMLV2-049)

  Scenario: export-sysml shall emit subsets and redefines on usages and round-trip multiplicity
    Given a native model using the feature
    When it is exported and the text re-ingested
    Then the SysML v2 text carries the construct and the re-ingested elements carry equal fields
```
