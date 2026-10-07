---
id: TC-TRS-SYSMLV2-023
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 viewpoint def becomes a native ViewpointDef with stakeholders and concerns, and a viewpoint usage maps onto a native View."
verifies:
  - REQ-TRS-SYSMLV2-021
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_views.rs
testFunctions:
  - viewpoint_def_lifts_stakeholders_and_concerns
  - viewpoint_usage_maps_onto_element_type_view
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_views.rs`; run with `cargo test -p syscribe-model --test sysmlv2_views`.

```gherkin
Feature: an ingested SysMLv2 viewpoint def becomes a native ViewpointDef with stakeholders and concerns (TC-TRS-SYSMLV2-023)

  Scenario: a viewpoint def lifts stakeholders and concerns
    Given a viewpoint def with stakeholder and concern members
    When the tool ingests the model
    Then a real ViewpointDef exists carrying stakeholders and concerns

  Scenario: a viewpoint usage maps onto View
    Given a viewpoint usage
    When the tool ingests the model
    Then a native View element is synthesized for it
```
