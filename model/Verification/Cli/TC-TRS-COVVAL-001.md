---
type: TestCase
id: TC-TRS-COVVAL-001
name: "W305 follows the coverage policy and E898 reports an invalid policy"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_validate.rs
verifies:
  - REQ-TRS-COVVAL-001
tags:
  - validation
---

```gherkin
Feature: coverage policy in validate

  Scenario: W305 and the rule
    Then the default names (rule: both), rollup silences it for a fully verified parent and not for one with an unverified leaf

  Scenario: E898
    Then an invalid table and a loosened integrity-rated parent are E898
```
