---
id: TC-TRS-SYSMLV2-028
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 case def/case becomes a native CaseDef/Case carrying subject, actors, objectives, result, isAbstract and doc."
verifies:
  - REQ-TRS-SYSMLV2-026
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_cases.rs
testFunctions:
  - a_case_def_and_usage_lift_subject_actors_objectives_result_and_doc
  - case_and_verification_nested_in_a_part_usage_body_stay_invisible
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_cases.rs`; run with `cargo test -p syscribe-model --test sysmlv2_cases`.

```gherkin
Feature: an ingested SysMLv2 case def/case becomes a native CaseDef/Case carrying subject (TC-TRS-SYSMLV2-028)

  Scenario: a case def and a case usage lift their body fields
    Given a case def and a case usage with subject, actor, objective and return members
    When the tool ingests the model
    Then a CaseDef and a Case exist carrying subject, actors, objectives, result and doc, with typedBy on the usage

  Scenario: a case nested in a part usage body is not reachable
    Given a case declared directly inside a part usage body
    When the tool ingests the model
    Then no element is synthesized for it
```
