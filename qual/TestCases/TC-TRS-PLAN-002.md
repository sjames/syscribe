---
id: TC-TRS-PLAN-002
type: TestCase
testLevel: L3
status: draft
title: "Verify TestPlan configuration binding: E606, escaping member W611 and duplicate-plan W616."
verifies:
  - REQ-TRS-PLAN-002
---

```gherkin
Feature: TestPlan configuration binding
  Scenario: a multi-config plan with resolvable configs is clean
    Given a TestPlan bound to two existing Configurations with a config-agnostic member
    When validate runs
    Then no E606 or W611 is raised

  Scenario: unresolvable configuration
    Given a TestPlan whose configurations entry names no Configuration
    When validate runs
    Then E606 is raised

  Scenario: escaping member
    Given a member TestCase active in none of the plan's bound configurations
    When validate runs
    Then W611 is raised

  Scenario: duplicate plan
    Given two TestPlans with an identical (configurations, scope) pair
    When validate runs
    Then W616 is raised
```
