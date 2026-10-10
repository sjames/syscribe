---
type: TestCase
id: TC-TRS-COVPOL-001
name: "coverage policy rules change the parent verdict and refuse loosening rated items"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_policy.rs
verifies:
  - REQ-TRS-COVPOL-001
tags:
  - cli
---

```gherkin
Feature: coverage policy

  Scenario: rollup rule
    Then a parent with all leaves verified and no direct test is complete under rollup and partial by default

  Scenario: selectors and order
    Then reqClass, requirementKind, status, tag and sil selectors match and the first matching rule wins

  Scenario: misconfiguration
    Then loosening an ASIL requirement or an invalid value exits 1 naming the cause
```
