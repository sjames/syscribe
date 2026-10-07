---
id: TC-TRS-SYSMLV2-025
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 concern def/concern becomes a native ConcernDef/Concern carrying subject and stakeholders, with exactly one of supertype or typedBy set."
verifies:
  - REQ-TRS-SYSMLV2-023
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_concerns.rs
testFunctions:
  - a_concern_def_becomes_a_real_concerndef_with_supertype
  - a_bare_concern_usage_becomes_a_real_concern_with_typed_by
  - subject_and_stakeholders_lift_from_a_concern_def_body
  - requires_and_assume_stay_unset_an_explicit_descope
  - a_concern_nested_inside_a_part_def_body_stays_invisible
  - a_concern_nested_inside_a_part_usage_body_stays_invisible
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_concerns.rs`; run with `cargo test -p syscribe-model --test sysmlv2_concerns`.

```gherkin
Feature: an ingested SysMLv2 concern def/concern becomes a native ConcernDef/Concern carrying subject and stakeholders (TC-TRS-SYSMLV2-025)

  Scenario: a concern def becomes a ConcernDef with supertype
    Given a concern def with a specialization clause
    When the tool ingests the model
    Then a real ConcernDef exists with supertype set and typedBy unset

  Scenario: a concern usage becomes a Concern with typedBy
    Given a bare concern usage typed by a concern def
    When the tool ingests the model
    Then a real Concern exists with typedBy set and supertype unset

  Scenario: subject and stakeholders lift from the body
    Given a concern def with a subject and stakeholder members
    When the tool ingests the model
    Then subject and stakeholders are populated and requires/assume stay unset
```
