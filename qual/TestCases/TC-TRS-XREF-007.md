---
id: TC-TRS-XREF-007
type: TestCase
testLevel: L3
status: draft
name: "Verify unresolved supertype/typedBy/subsets/redefines/satisfies raise E110–E114; every §11.5 resolution form stays clean; root-name hint applies; [repos] models use E512."
verifies:
  - REQ-TRS-XREF-007
---

```gherkin
Feature: unresolved structural cross-references are reported (GH #125)
  Scenario: every structural field with a missing target raises its own code
    Given a model whose supertype, typedBy (usage and inline feature), subsets, redefines and satisfies name missing elements
    When the model is validated
    Then E110, E111 twice, E112, E113 and E114 are raised and the exit code is non-zero

  Scenario: references resolving by any §11.5 form raise nothing
    Given references by relative scope, ./ sibling, import, alias, inline feature, standard-library name and stable id
    When the model is validated
    Then none of E110-E114 is raised and the exit code is zero

  Scenario: a root-prefixed supertype carries the root-name hint
    Given a supertype written as <RootName>::Lib::Base where Lib::Base exists
    When the model is validated
    Then E110 is raised with a hint naming Lib::Base

  Scenario: a [repos] model reports an unresolved reference once, as E512
    Given a [repos] model with one supertype resolving in the peer and one resolving nowhere
    When the model is validated
    Then the peer reference raises nothing, the missing one raises E512, and no E110 is raised
```
