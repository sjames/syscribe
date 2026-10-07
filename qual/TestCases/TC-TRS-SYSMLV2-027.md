---
id: TC-TRS-SYSMLV2-027
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 enum def/enum becomes a native EnumerationDef/Enumeration carrying values, supertype and typedBy."
verifies:
  - REQ-TRS-SYSMLV2-025
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_enums.rs
testFunctions:
  - an_enum_def_lifts_values_and_supertype_with_no_doc
  - an_enum_literal_with_an_initializer_keeps_only_its_name
  - a_named_enum_usage_lifts_typed_by_and_doc
  - an_enum_def_and_usage_nested_in_a_part_usage_body_are_also_reachable
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_enums.rs`; run with `cargo test -p syscribe-model --test sysmlv2_enums`.

```gherkin
Feature: an ingested SysMLv2 enum def/enum becomes a native EnumerationDef/Enumeration carrying values (TC-TRS-SYSMLV2-027)

  Scenario: an enum def lifts values and supertype
    Given an enum def with literals, one with an initializer
    When the tool ingests the model
    Then a real EnumerationDef exists whose values are name-only entries

  Scenario: an enum usage lifts typedBy
    Given a named enum usage
    When the tool ingests the model
    Then a real Enumeration exists with typedBy and doc

  Scenario: nested enums in a part usage are reachable
    Given an enum def and usage declared in a part usage body
    When the tool ingests the model
    Then both become real elements
```
