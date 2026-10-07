---
id: TC-TRS-SYSMLV2-029
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 analysis def/analysis becomes a native AnalysisCaseDef/AnalysisCase and a verification def/verification becomes a native VerificationCaseDef/VerificationCase, sharing the case field extraction."
verifies:
  - REQ-TRS-SYSMLV2-027
  - REQ-TRS-SYSMLV2-028
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_cases.rs
testFunctions:
  - an_analysis_def_is_reachable_at_all_three_nesting_levels
  - a_verification_def_with_multiple_returns_takes_the_first_typed_one
  - case_and_verification_nested_in_a_part_usage_body_stay_invisible
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_cases.rs`; run with `cargo test -p syscribe-model --test sysmlv2_cases`.

```gherkin
Feature: an ingested SysMLv2 analysis def/analysis becomes a native AnalysisCaseDef/AnalysisCase and a verification def/verification becomes a native VerificationCaseDef/VerificationCase (TC-TRS-SYSMLV2-029)

  Scenario: an analysis def is reachable at package, part def and part usage level
    Given analysis defs at all three nesting levels
    When the tool ingests the model
    Then an AnalysisCaseDef is synthesized at each level

  Scenario: a verification def takes the first typed return
    Given a verification def with several return declarations
    When the tool ingests the model
    Then a VerificationCaseDef exists whose result is the first typed return

  Scenario: a verification nested in a part usage is not reachable
    Given a verification declared directly inside a part usage body
    When the tool ingests the model
    Then no element is synthesized for it
```
