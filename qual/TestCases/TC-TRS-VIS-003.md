---
id: TC-TRS-VIS-003
type: TestCase
testLevel: L2
status: active
name: "Verify that a Diagram with a subject and no shapes is derived through the real walker and validator, that include/exclude filter it and a stray entry or filters on a manifest diagram are W417, that a wrong subject type is W418 and draws nothing, that pins apply through deterministic ids, and that generated refs raise no W402/W403."
verifies:
  - REQ-TRS-VIS-003
sourceFile: repo:crates/syscribe-model/tests/vis_derive.rs
testFunctions:
  - derived_bdd_of_a_package_matches_its_golden_ir
  - derived_ibd_of_a_partdef_matches_its_golden_ir
  - pins_survive_regeneration_through_deterministic_ids
  - include_and_exclude_filter_and_a_stray_entry_is_w417
  - filters_on_a_manifest_diagram_are_w417_and_ignored
  - a_subject_of_the_wrong_type_is_w418_and_draws_nothing
  - a_derived_diagram_raises_no_w402_for_its_generated_refs
tags:
  - diagram
  - visualisation
  - validation
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive.rs`; run with
`cargo test -p syscribe-model --test vis_derive`. Each test writes the fixture model (a `Sys`
package with an abstract base, two definitions with ports, a port definition, a connection
definition and a composed `PowerSystem`) plus one `Diagram` to a temp directory, runs the real
walker and validator, and builds the diagram's IR through `vis::build_graph`. The generated IR
is pinned by golden JSON snapshots under `crates/syscribe-model/tests/vis_snapshots/derived/`
(refresh with `SYSCRIBE_UPDATE_SNAPSHOTS=1`).

```gherkin
Feature: a Diagram with a subject and no shapes is derived from the model (TC-TRS-VIS-003)

  Scenario: a derived BDD of a package matches its golden IR
    Given a BDD diagram with subject Sys and no shapes block
    When the model is validated and the IR is built
    Then there is no W417, W418 or E405
    And the IR has six blocks, two inheritance, three composition and one association edge
    And it matches the sys_bdd golden snapshot

  Scenario: a derived IBD of a PartDef matches its golden IR
    Given an IBD diagram with subject Sys::PowerSystem and no shapes block
    When the model is validated and the IR is built
    Then the single root is a boundary with three blocks and four ports
    And the edges are a connection followed by a binding
    And it matches the power_ibd golden snapshot

  Scenario: pins survive regeneration through deterministic ids
    Given a derived IBD with a layout entry keyed s-sys-powersystem-engine
    When the model is validated and the IR is built
    Then no W416 is raised and that node is pinned

  Scenario: include and exclude filter the view and a stray entry is W417
    Given a derived BDD with include [Engine, Motor, Ghost] and exclude [Motor]
    When the model is validated and the IR is built
    Then only Sys::Engine is on the diagram
    And exactly one W417 names Ghost

  Scenario: filters on a manifest diagram are W417 and ignored
    Given a diagram with a shapes block of two shapes and an include list
    When the model is validated and the IR is built
    Then both shapes are in the IR unfiltered
    And exactly one W417 is raised

  Scenario: a subject of the wrong type is W418 and draws nothing
    Given an IBD diagram whose subject is the Sys package
    When the model is validated and the IR is built
    Then the IR has no nodes
    And exactly one W418 names the Package type

  Scenario: a derived diagram raises no W402 for its generated refs
    Given a derived IBD whose ports reference inline features
    When the model is validated
    Then no W402 and no W403 is raised for the diagram
```
