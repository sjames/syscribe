---
id: TC-TRS-VIS-020
type: TestCase
testLevel: L2
status: active
name: "Verify the Requirement generator draws the requirements under a package, RequirementDef or Requirement subject with id/status compartments, derive, refine, containment, satisfy and verify edges from real-type context nodes, applies include/exclude by qualified name, id or short name with W417, reports a wrong subject as W418, and is accepted by the Mermaid, SVG and PlantUML writers."
verifies:
  - REQ-TRS-VIS-020
sourceFile: repo:crates/syscribe-model/tests/vis_derive_trace.rs
testFunctions:
  - derived_requirement_diagram_of_a_package_matches_its_golden_ir
  - requirement_def_and_requirement_subjects_root_the_tree
  - requirement_filters_apply_to_requirements_and_context_nodes_and_a_stray_entry_is_w417
  - requirement_subject_of_the_wrong_type_is_w418_and_draws_nothing
  - requirement_writers_accept_the_derived_graph
tags:
  - diagram
  - visualisation
  - requirement
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive_trace.rs`; run with
`cargo test -p syscribe-model --test vis_derive_trace`. Each test writes the fixture model
(a `Reqs` package with a top requirement, a `RequirementDef` owning two derived children, an
`Arch` package with a satisfying `PartDef`, a `Tests` package with a verifying `TestCase`) to a
temp directory and runs the real walker and validator on it; the generated IR of the package
subject is pinned by `tests/vis_snapshots/derived/reqs.json`. The generator's own unit tests
(`crates/syscribe-model/src/vis/derive/requirement.rs`) cover the same rules on the shared
in-memory fixture.

```gherkin
Feature: derived Requirement diagrams (TC-TRS-VIS-020)

  Scenario: a package subject
    Given a Requirement diagram whose subject is the Reqs package and no shapes
    When the model is walked and the diagram built
    Then there is one requirement node per Requirement and RequirementDef at any depth
    And each carries id = … and status = … compartment lines when present
    And derive edges run child → parent, a refine edge runs refining → refined
    And containment edges run from the RequirementDef to the requirements it owns
    And a satisfy edge runs from a part def block and a verify edge from a test case node
    And the IR equals the golden snapshot

  Scenario: a RequirementDef or Requirement subject
    Given the subject is the Safety RequirementDef
    Then only it and its owned requirements are drawn, with no derive edge to the outside parent
    Given the subject is a requirement named by stable id
    Then only that requirement and its satisfier are drawn

  Scenario: filters
    Given include names a requirement by id, one by short name, a test case and a stranger
    Then the named requirements are kept, the test case draws nothing and one W417 names the stranger
    Given exclude names the satisfier and the RequirementDef
    Then they and their satisfy and containment edges are gone

  Scenario: a wrong subject
    Given the subject is a PartDef
    Then W418 is raised and the diagram is empty

  Scenario: writers
    Given the derived graph of the package
    Then Mermaid is a classDiagram with requirement classes, members and keyword dependencies
    And SVG succeeds with dashed keyword-labelled edges and header bands
    And PlantUML lists each requirement with its id and status body and no compartment class
```
