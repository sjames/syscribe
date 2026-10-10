---
type: TestCase
id: TC-TRS-REQKIND-001
name: "process, regulatory and deliverable requirements skip W300/W302"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/requirement_kinds.rs
verifies:
  - REQ-TRS-REQKIND-001
tags:
  - requirements
---

```gherkin
Feature: non-allocatable requirement kinds

  Scenario: accepted kinds
    Then process, regulatory and deliverable raise no E022 and an unknown kind still does

  Scenario: W300
    Given an approved leaf of each new kind with no satisfier
    Then W300 is not raised, but is raised for kind system

  Scenario: W302
    Given an implemented leaf of a new kind with reqDomain system
    Then W302 is not raised

  Scenario: verification coverage unchanged
    Given an approved process requirement with no active test
    Then W002 is still raised
```
