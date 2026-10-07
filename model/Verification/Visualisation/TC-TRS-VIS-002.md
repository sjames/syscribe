---
id: TC-TRS-VIS-002
type: TestCase
testLevel: L2
status: active
name: "Verify the validator reports a malformed shapes/edges/layout manifest as E405 and a stale layout pin as W416 through the real walker and validator, and never for a Mermaid-kind diagram."
verifies:
  - REQ-TRS-VIS-002
sourceFile: repo:crates/syscribe-model/tests/vis_manifest_validation.rs
testFunctions:
  - string_shorthand_shapes_are_well_formed_and_reach_the_ir
  - unknown_shape_kind_is_one_e405_naming_the_shape_and_the_rest_still_validates
  - stale_layout_key_is_w416
  - shapes_sequence_of_scalars_is_e405
  - mermaid_kind_has_no_ir_and_no_e405
tags:
  - diagram
  - visualisation
  - validation
---

Hosted integration tests in `crates/syscribe-model/tests/vis_manifest_validation.rs`; run with
`cargo test -p syscribe-model --test vis_manifest_validation`. Each test writes a small model to
a temp directory and runs the real walker and validator on it.

```gherkin
Feature: a malformed diagram manifest is E405, a stale pin is W416 (TC-TRS-VIS-002)

  Scenario: the string shorthand is well-formed
    Given a BDD diagram whose shapes are written as id: Qualified::Name
    When the model is validated
    Then no E405 or W416 is raised and the shapes are in the IR

  Scenario: an unknown kind names its shape and spares the rest
    Given a diagram with one shape of kind gizmo among well-formed shapes
    When the model is validated
    Then exactly one E405 names that shape and the other shapes still build

  Scenario: a stale layout key
    Given a layout entry whose key names no shape of the diagram
    When the model is validated
    Then W416 is raised for that key

  Scenario: shapes written as a list of scalars
    Given a shapes value that is a sequence of strings
    When the model is validated
    Then E405 names the shapes section

  Scenario: a Mermaid-kind diagram has no manifest to check
    Given a diagramKind Mermaid diagram with a malformed shapes value and a mermaid block
    When the model is validated
    Then no E405 is raised and the element has no IR
```
