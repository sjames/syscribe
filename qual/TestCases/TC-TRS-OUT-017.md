---
id: TC-TRS-OUT-017
type: TestCase
testLevel: L3
status: draft
name: "Verify the impact command: downstream reaches derived children / satisfying elements / verifying tests; upstream traces back to the safety goal; --kinds and --depth filter; --format json/dot are well-formed."
verifies:
  - REQ-TRS-OUT-017
---

Verify change impact analysis against a small traceability chain
(SG → REQ → leaf REQ → {PartImp, TC}).

```gherkin
Feature: Change impact analysis (§17)

  Scenario: downstream reaches the full chain
    Given a safety goal with a derived requirement chain
    When `impact SG-IMP-001` is run
    Then REQ-IMP-000, REQ-IMP-LEAF-001, PartImp and TC-IMP-001 are reported

  Scenario: upstream traces back to the safety goal
    When `impact REQ-IMP-LEAF-001 --direction upstream` is run
    Then SG-IMP-001 is reached

  Scenario: --kinds restricts the followed edges
    When `impact REQ-IMP-LEAF-001 --kinds verifies` is run
    Then the verifying test is shown but the satisfying part is not

  Scenario: --depth limits the hops
    When `impact SG-IMP-001 --depth 1` is run
    Then only the depth-1 child is reported

  Scenario: --format json matches the schema
    When `impact SG-IMP-001 --format json` is run
    Then the output has root and nodes with via labels

  Scenario: --format dot is valid Graphviz
    When `impact SG-IMP-001 --format dot` is run
    Then the output is a digraph

  Scenario: stable id and qualified name both work as the root
    When `impact imp::PartImp --direction upstream` is run
    Then the leaf requirement is reached
```
