---
id: TC-TRS-VIS-009
type: TestCase
testLevel: L3
status: active
name: "Verify diagram export writes PlantUML, Mermaid or SVG for one Diagram element, refuses an unknown element, a non-Diagram, an unpinned SVG and a bad --format with the specified messages and exit 1, and that MCP render_diagram accepts format mermaid."
verifies:
  - REQ-TRS-VIS-009
sourceFile: repo:crates/syscribe/tests/diagram_export.rs
testFunctions:
  - unknown_element_is_an_error_with_nothing_on_stdout
  - a_non_diagram_element_is_refused
  - mermaid_goes_to_stdout_with_a_ref_per_node
  - plantuml_is_the_default_format
  - out_writes_the_file_and_creates_parents
  - svg_is_refused_for_an_unpinned_diagram
  - svg_draws_a_fully_pinned_diagram_with_sysml_attributes
  - bad_format_is_a_usage_error_naming_the_valid_values
  - an_unknown_option_is_rejected_before_the_model_loads
  - only_export_is_a_diagram_subcommand
tags:
  - diagram
  - visualisation
  - cli
---

Black-box CLI tests in `crates/syscribe/tests/diagram_export.rs`, run with
`cargo test -p syscribe --test diagram_export`, spawning the built binary against the fixture
model under `crates/syscribe/tests/fixtures/model` (`Diagrams::FxBlock` is an unpinned manifest
BDD, `Diagrams::FxPinned` a fully pinned one). The MCP side (`render_diagram {format: mermaid}`
and the `svg` refusal) is covered by `render_diagram_generates_mermaid_from_the_ir_on_request`
and `render_diagram_svg_needs_a_fully_pinned_diagram` in `crates/syscribe/tests/mcp_diagrams.rs`;
the Mermaid writer's golden snapshots by `TC-TRS-VIS-010`'s source file.

```gherkin
Feature: diagram export (TC-TRS-VIS-009)

  Scenario: an unknown element
    When the tool runs diagram export Nope::Missing --format mermaid
    Then it exits 1, prints nothing on stdout and "error: element 'Nope::Missing' not found" on stderr

  Scenario: a non-Diagram element
    When the tool runs diagram export Parts::Base --format mermaid
    Then it exits 1 with "error: 'Parts::Base' is not a Diagram"

  Scenario: Mermaid to stdout
    When the tool runs diagram export Diagrams::FxBlock --format mermaid
    Then stdout is a classDiagram with a %% ref: line per node and <|-- inheritance

  Scenario: PlantUML is the default
    When the tool runs diagram export Diagrams::FxBlock
    Then stdout starts with @startuml

  Scenario: --out writes the file
    When the tool runs diagram export with --out <dir>/nested/deeper/FxBlock.mmd
    Then the file is written, its parents created, and stdout is empty

  Scenario: svg refused when unpinned
    When the tool runs diagram export Diagrams::FxBlock --format svg
    Then it exits 1 with "error: 'Diagrams::FxBlock' is not fully pinned — open it in the browser and use Pin all, or export plantuml/mermaid"

  Scenario: svg drawn when fully pinned
    When the tool runs diagram export Diagrams::FxPinned --format svg
    Then stdout is an SVG with the sysml namespace, a sysml:ref per shape and sysml:source/target on the edge

  Scenario: a bad --format
    When the tool runs diagram export Diagrams::FxBlock --format png
    Then it exits 1 with a usage error naming plantuml, mermaid, svg

  Scenario: only export exists
    When the tool runs diagram list
    Then it exits 1, prints nothing on stdout and names the unrecognized subcommand
```
