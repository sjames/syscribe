---
id: TC-TRS-SYSMLV2-098
type: TestCase
testLevel: L3
status: active
name: "Verify bare root-level definitions and usages merge under the submodel's anchor package and round-trip through export."
verifies:
  - REQ-TRS-SYSMLV2-098
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_root_members.rs
testFunctions:
  - root_level_definitions_and_usages_merge_under_the_anchor
  - root_level_members_merge_across_files_and_resolve_like_package_members
  - root_level_prefix_metadata_doc_and_alias_lift_like_a_package_body
  - root_level_members_count_under_their_package_kind_in_w543
  - root_level_members_export_as_direct_members_of_the_anchor_and_round_trip
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_root_members.rs`; run with `cargo test -p syscribe-model --test sysmlv2_root_members`.

```gherkin
Feature: SysMLv2 bare root-level members (TC-TRS-SYSMLV2-098)

  Scenario: Root-level definitions and usages become the anchor's members
    Given a sysmlSubmodel package whose .sysml files declare part def, part, requirement def, attribute def, item def, action def and state def outside every package
    When the model is walked
    Then each becomes an element with qualified name <anchor>::<name>, nested body members under it, and W543 is not raised

  Scenario: Root-level members merge across files and resolve like package members
    Given one file declaring part def Engine and requirement def Thrust at its root, and another declaring part engine : Engine, satisfy Thrust by engine and a package using Engine
    When the model is walked
    Then engine satisfies Thrust, the nested usage is typed by Engine, and W543 is not raised

  Scenario: Prefix, metadata, doc and alias forms lift like a package body
    Given a root-level #Tag prefix, metadata usage, @Tag about applications, doc comment and alias
    When the model is walked
    Then the prefixed and targeted definition carries the metadata, the anchor carries the usage, the dangling about and the alias, its documentation gains the doc, and W543 counts only the dangling about

  Scenario: Export writes root-level members as direct members of the anchor and the text re-ingests
    Given root-level elements and a package under the anchor
    When export-sysml is run for the anchor and its output is ingested under a fresh anchor
    Then the members appear one level inside package <anchor> and the re-ingested elements match
```
