---
type: TestCase
id: TC-TRS-W015SPLIT-001
name: "W015 names whether the leaves below a parent are verified"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/w015_split.rs
verifies:
  - REQ-TRS-W015SPLIT-001
tags:
  - validation
---

```gherkin
Feature: W015 message split

  Scenario: parent with verified leaves
    Then W015 says it is covered through its children only

  Scenario: parent with an unverified leaf and a plain leaf
    Then W015 says not every leaf is verified and a leaf's message is unchanged
```
