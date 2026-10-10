---
type: TestCase
id: TC-TRS-NOISE-001
name: "W015 grouped per requirement and validate --summary"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/validate_noise.rs
verifies:
  - REQ-TRS-NOISE-001
tags:
  - validation
---

```gherkin
Feature: validation noise

  Scenario: W015 grouped
    Given 2 requirements active in 3 configurations with no tests
    Then exactly 2 W015 findings are raised, each naming all 3 configurations

  Scenario: partially covered
    Given a requirement covered in one of two configurations
    Then its W015 names only the uncovered configuration

  Scenario: summary
    When validate --summary is run
    Then a per-code count table is printed and per-finding rows are not
```
