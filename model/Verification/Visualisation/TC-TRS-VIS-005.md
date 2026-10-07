---
id: TC-TRS-VIS-005
type: TestCase
testLevel: L2
status: active
name: "Verify the IBD generator derives the boundary, owned part usages, ports with direction and connection/binding edges from a PartDef subject, carries a Part subject's definition ports, drops an excluded usage with its edges without inventing a port, flags an include naming no usage as W417, reports a wrong subject type as W418, and draws no edge for a chain to an unknown port."
verifies:
  - REQ-TRS-VIS-005
sourceFile: repo:crates/syscribe-model/src/vis/derive/ibd.rs
testFunctions:
  - partdef_subject_yields_boundary_parts_ports_and_edges
  - part_usage_subject_carries_its_definitions_ports
  - exclude_drops_a_usage_and_its_edges_and_the_generator_never_invents_a_port
  - include_naming_no_usage_is_w417
  - wrong_subject_type_is_w418
  - a_chain_to_an_unknown_port_produces_no_edge
tags:
  - diagram
  - visualisation
  - ibd
---

Unit tests in `crates/syscribe-model/src/vis/derive/ibd.rs`; run with
`cargo test -p syscribe-model vis::derive::ibd`. Each test derives an `IBD` diagram against the
shared in-memory derive fixture (`vis::derive::testkit::model`), whose `PowerSystem` declares
inline `engine` and `motor [2]` usages, a boundary port `mainOut`, a child `aux` part, a
`PowerLink` connection `engine.powerOut → motor.powerIn` and a binding
`motor.powerIn = mainOut`.

```gherkin
Feature: the IBD generator derives the boundary, parts, ports and edges from a PartDef or Part subject (TC-TRS-VIS-005)

  Scenario: a PartDef subject yields the boundary, parts, ports and edges
    Given an IBD diagram with subject Sys::PowerSystem
    When the diagram is derived
    Then there are no issues
    And s-sys-powersystem is a boundary with stereotype "part def"
    And its port mainOut is a Port child with direction out
    And the blocks are exactly "engine : Engine", "motor : Motor [2]" and "aux : Motor", all parented to the boundary
    And engine.powerOut is a port of the engine block with direction out and element ref Sys::PowerSystem::engine::powerOut
    And aux.powerIn has direction in
    And the edges are a Connection engine.powerOut → motor.powerIn labelled PowerLink followed by a Binding motor.powerIn → mainOut
    And the connection edge id is e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin
    And the layout hints are hierarchical

  Scenario: a Part subject carries its definition's ports
    Given an IBD diagram with subject Sys::PowerSystem::aux
    When the diagram is derived
    Then the subject is a boundary
    And its powerIn port from the Motor definition has direction in

  Scenario: exclude drops a usage and its edges and the generator never invents a port
    Given an IBD diagram with subject Sys::PowerSystem and exclude [motor]
    When the diagram is derived
    Then there are no issues
    And the motor block is absent
    And there are no edges because both edges touched motor.powerIn

  Scenario: an include naming no usage is W417
    Given an IBD diagram with subject Sys::PowerSystem and include [engine, turbine]
    When the diagram is derived
    Then exactly one block remains
    And exactly one W417 names turbine

  Scenario: a wrong subject type is W418
    Given an IBD diagram whose subject is the Sys package
    When the diagram is derived
    Then the graph has no nodes
    And a W418 names the Package type

  Scenario: a chain to an unknown port produces no edge
    Given a PowerSystem whose only connection runs from engine.powerOut to ghost.powerIn
    When the diagram is derived
    Then there are no issues from the generator
    And there are no edges
```
