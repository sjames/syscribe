---
type: TestCase
id: TC-TRS-BDDX-001
name: "derived BDD pulls in cross-package composition targets with a depth limit"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/src/vis/derive/bdd.rs
verifies:
  - REQ-TRS-BDDX-001
tags:
  - vis
---

```gherkin
Feature: derived BDD across packages

  Scenario: package subject
    Then a part typed by a definition in another package adds an external block and a composition edge

  Scenario: definition subject and depth
    Then a PartDef subject shows its composed blocks and depth 0 / 2 change how far composition is followed

  Scenario: qualified include
    Then include accepts a qualified name relative to the subject without W417
```
