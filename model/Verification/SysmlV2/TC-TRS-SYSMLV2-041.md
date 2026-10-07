---
id: TC-TRS-SYSMLV2-041
type: TestCase
testLevel: L3
status: active
name: "Verify determinism and the parse-back round trip for a native fixture and the sysmlv2-submodel example."
verifies:
  - REQ-TRS-SYSMLV2-041
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export.rs
testFunctions:
  - export_is_deterministic
  - scope_limits_the_subtree_and_unknown_scope_errors
  - native_model_round_trips_through_ingestion
  - example_submodel_round_trips_through_ingestion
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-041)

  Scenario: Tool shall produce deterministic SysML v2 output that re-parses through ingestion with kinds and qnames intact
    Given a native fixture and examples/sysmlv2-submodel
    When each is exported and the text parsed back by the SysML v2 ingestion
    Then no parse failure occurs and supported qnames and kinds are preserved, and two exports are byte-identical
```
