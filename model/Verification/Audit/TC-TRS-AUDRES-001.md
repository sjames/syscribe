---
type: TestCase
id: TC-TRS-AUDRES-001
name: "audit shows verification results and fails on a failing goal or approved requirement"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/audit_results.rs
verifies:
  - REQ-TRS-AUDRES-001
tags:
  - audit
---

```gherkin
Feature: audit and verification results

  Scenario: failing goal fails the verdict
    Given a SafetyGoal whose supporting TestCase fails in the ingested results
    Then audit lists the goal as failing and the verdict is FAIL with a reason naming it

  Scenario: verification section counts
    Then tests, goals and plans are counted by verdict

  Scenario: no results
    Then the verification section is null and no failing-goal reason appears

  Scenario: approved requirement with failing verifier
    Then W312 appears in the verdict reasons by default
```
