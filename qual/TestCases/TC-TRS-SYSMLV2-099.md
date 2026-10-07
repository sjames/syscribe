---
id: TC-TRS-SYSMLV2-099
type: TestCase
testLevel: L3
status: active
name: "Verify a #T prefix on a usage applies metadata to that usage wherever the usage is declared, in both parser shapes, and round-trips through export."
verifies:
  - REQ-TRS-SYSMLV2-099
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_audit_gaps.rs
testFunctions:
  - a_prefix_on_a_usage_inside_a_definition_body_applies_metadata_in_both_parser_shapes
  - a_prefix_on_a_package_level_or_root_level_usage_applies_metadata
  - a_prefix_on_a_member_that_becomes_no_element_is_dropped
  - a_prefixed_usage_round_trips_through_export
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_audit_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_audit_gaps`.

```gherkin
Feature: SysMLv2 usage #T prefixes (TC-TRS-SYSMLV2-099)

  Scenario: A prefix inside a definition body applies metadata in both parser shapes
    Given a part def whose body prefixes part, attribute, action, port, item and occurrence usages with #Tag, plus a part usage and an action def doing the same
    When the submodel is ingested
    Then every prefixed usage carries exactly one metadata entry typed by the ingested Tag def, the holder's own @Tag; stays on the holder, an unprefixed sibling carries none, and W543 is not raised

  Scenario: A prefix on a package-level or root-level usage applies metadata
    Given #Tag part p : X; at the file root and #Tag part/attribute usages and a #Tag part def in a package
    When the submodel is ingested
    Then each prefixed member carries the entry and the unprefixed definition carries none

  Scenario: A prefix on a member that becomes no element is dropped
    Given #Tag actor Act; followed by part def A
    When the submodel is ingested
    Then A carries no metadata and W543 counts only the actor

  Scenario: A prefixed usage round-trips through export
    Given the ingested prefixed usages
    When export-sysml writes the package and its output is re-ingested
    Then every usage body carries @<Tag>; and every re-ingested usage carries the same single entry as its holder's own annotation
```
