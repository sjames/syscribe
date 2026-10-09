---
id: TC-TRS-SYSMLV2-026
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 flow def/flow becomes a native FlowDef/Flow, and a flow nested in a part also lifts its endpoints onto the owning part flowConnections."
verifies:
  - REQ-TRS-SYSMLV2-024
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_flows.rs
testFunctions:
  - a_flow_def_becomes_a_real_flowdef_with_supertype_and_no_ends_or_itemtype
  - a_named_top_level_flow_usage_becomes_a_real_flow_with_item_type
  - an_anonymous_flow_nested_in_a_part_def_becomes_a_flow_connections_entry_only
  - a_named_flow_nested_in_a_part_usage_produces_both_an_element_and_an_entry
  - succession_flow_kind_lifts_as_succession
  - a_two_segment_flow_endpoint_keeps_its_full_path_without_w542
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_flows.rs`; run with `cargo test -p syscribe-model --test sysmlv2_flows`.

```gherkin
Feature: an ingested SysMLv2 flow def/flow becomes a native FlowDef/Flow (TC-TRS-SYSMLV2-026)

  Scenario: a flow def and a named flow usage become native elements
    Given a flow def and a named top-level flow usage with an item type
    When the tool ingests the model
    Then a real FlowDef and a real Flow with itemType exist

  Scenario: nested flows lift onto flowConnections
    Given an anonymous flow in a part def and a named flow in a part usage
    When the tool ingests the model
    Then the owning part carries flowConnections entries, and the named one is also its own element

  Scenario: succession flows and truncated endpoints
    Given a succession flow and a flow whose endpoint chain is unresolvable
    When the tool ingests and validates the model
    Then the kind is succession and the full dotted path is kept and no W542 is raised (#206)
```
