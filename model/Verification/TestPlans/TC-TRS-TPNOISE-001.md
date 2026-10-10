---
type: TestCase
id: TC-TRS-TPNOISE-001
name: "W616 uses member overlap and W615 is aggregated per plan"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/testplan_warning_noise.rs
verifies:
  - REQ-TRS-TPNOISE-001
  - REQ-TRS-TPNOISE-002
tags:
  - testplan
---

```gherkin
Feature: TestPlan warning noise

  Scenario: distinct plans sharing a scope are not redundant
    Given three plans with scope security and disjoint members
    Then W616 is not raised

  Scenario: plans with identical members are redundant
    Given two plans with scope smoke and the same members
    Then W616 is raised once

  Scenario: a subset plan is redundant
    Given a plan whose members are a subset of another plan's in the same bucket
    Then W616 is raised

  Scenario: W615 once per plan
    Given two approved plans that both contain two failing functions
    Then exactly two W615 are raised, each naming both functions
```
