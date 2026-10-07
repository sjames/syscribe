---
id: TC-TRS-VIS-004
type: TestCase
testLevel: L2
status: active
name: "Verify the BDD generator lists the subject's definitions with compartments, inheritance, composition and association edges, includes a definition subject itself with edges only between diagram nodes, applies include/exclude and flags unknown entries as W417, and reports a wrong subject type as W418 with an empty graph."
verifies:
  - REQ-TRS-VIS-004
sourceFile: repo:crates/syscribe-model/src/vis/derive/bdd.rs
testFunctions:
  - package_subject_lists_definitions_with_compartments_and_edges
  - definition_subject_includes_itself_and_edges_only_between_diagram_nodes
  - include_and_exclude_filter_members_and_flag_unknown_entries
  - wrong_subject_type_is_w418_and_empty
tags:
  - diagram
  - visualisation
  - bdd
---

Unit tests in `crates/syscribe-model/src/vis/derive/bdd.rs`; run with
`cargo test -p syscribe-model vis::derive::bdd`. Each test derives a `BDD` diagram against the
shared in-memory derive fixture (`vis::derive::testkit::model`: a `Sys` package with an
abstract `Base`, `Engine` and `Motor` specialising it, a `PowerPort`, a `PowerLink` connection
definition, a composed `PowerSystem` with inline usages and a child `aux` part, and an
`ActionDef` that must never appear).

```gherkin
Feature: the BDD generator derives blocks and edges from a Package or definition subject (TC-TRS-VIS-004)

  Scenario: a package subject lists its definitions with compartments and edges
    Given a BDD diagram with subject Sys
    When the diagram is derived
    Then there are no issues
    And the blocks are exactly Base, Engine, Motor, PowerLink, PowerPort and PowerSystem
    And the ActionDef Startup is not on the diagram
    And Base is abstract with stereotype "part def" and PowerLink has stereotype "connection def"
    And Engine's compartment lists "mass : Real [kg]" and "port powerOut : PowerPort (out)"
    And PowerSystem's compartment lists only its port
    And the edges are two inheritance edges to Base, three composition edges labelled engine, "motor [2]" and aux, and one association labelled PowerLink
    And the composition edge id is e-composition-s-sys-powersystem-s-sys-motor-motor

  Scenario: a definition subject includes itself and draws edges only between diagram nodes
    Given a BDD diagram with subject Sys::PowerSystem
    When the diagram is derived
    Then there is exactly one block
    And there are no edges because Engine and Motor are not on the diagram

  Scenario: include and exclude filter members and flag unknown entries
    Given a BDD diagram with subject Sys and include [Engine, Sys::Base, Ghost]
    When the diagram is derived
    Then the blocks are Base and Engine and only their inheritance edge survives
    And exactly one W417 names the include entry Ghost
    Given a BDD diagram with subject Sys and exclude [Sys::Base]
    When the diagram is derived
    Then Base is absent, no inheritance edge remains and there are no issues

  Scenario: a wrong subject type is W418 and the graph is empty
    Given a BDD diagram whose subject is the ActionDef Sys::Startup
    When the diagram is derived
    Then the graph has no nodes
    And exactly one W418 names the ActionDef type
```
