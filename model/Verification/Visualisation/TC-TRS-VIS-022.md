---
id: TC-TRS-VIS-022
type: TestCase
testLevel: L2
status: active
name: "Verify the Allocation generator collects every allocation pair form under a package, AllocationDef or Allocation subject into a logical and a physical swimlane of real-type blocks with one labelled «allocate» edge per pair, draws an unresolved end dashed and a two-sided element once per lane, applies include/exclude to the ends with W417, reports a wrong subject as W418, and is accepted by the Mermaid and SVG writers."
verifies:
  - REQ-TRS-VIS-022
sourceFile: repo:crates/syscribe-model/tests/vis_derive_trace.rs
testFunctions:
  - derived_allocation_diagram_of_a_package_matches_its_golden_ir
  - allocation_filters_apply_to_the_end_elements_and_a_stray_entry_is_w417
  - allocation_subject_of_the_wrong_type_is_w418_and_draws_nothing
  - allocation_writers_accept_the_derived_graph
tags:
  - diagram
  - visualisation
  - allocation
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive_trace.rs`; run with
`cargo test -p syscribe-model --test vis_derive_trace`. Each test writes the fixture model
(an `Allocations` package with an `Allocation` element using `features:` pairs, one of them
to an unresolved target, an `AllocationDef` with `allocations:` and a `PartDef` with its own
`allocatedTo:`, over an `Arch` package of parts and an action) to a temp directory and runs the
real walker and validator on it; the generated IR of the package subject is pinned by
`tests/vis_snapshots/derived/alloc.json`. The generator's own unit tests
(`crates/syscribe-model/src/vis/derive/allocation.rs`) cover the same rules, plus the
top-level `allocatedFrom:`/`allocatedTo:` form, on the shared in-memory fixture.

```gherkin
Feature: derived Allocation diagrams (TC-TRS-VIS-022)

  Scenario: a package subject
    Given an Allocation diagram whose subject is the Allocations package and no shapes
    When the model is walked and the diagram built
    Then there are two swimlanes, <subject id>-logical and <subject id>-physical
    And every source is a block in the logical lane and every target one in the physical lane
    And each block carries its real type's stereotype
    And the unresolved end is a dashed block labelled by its reference text
    And the element that is both a source and a target appears once in each lane
    And there is one allocate edge per pair, labelled by its usage name when it has one
    And the IR equals the golden snapshot

  Scenario: filters
    Given include names ends by short name, qualified name and reference text, plus a stranger
    Then only the named ends are drawn, only the edge joining two kept ends survives, and one W417 names the stranger
    Given exclude names a source
    Then it and its edges are gone

  Scenario: subjects
    Given the subject is a PartDef
    Then W418 is raised and the diagram is empty
    Given the subject is an AllocationDef
    Then its own pairs are drawn

  Scenario: writers
    Given the derived graph of the package
    Then Mermaid is a flowchart LR with one subgraph per lane and -.-> allocate edges
    And SVG succeeds with swimlane groups, 8,4-dashed «allocate» edges and a dashed unresolved block
    And PlantUML declines the kind
```
