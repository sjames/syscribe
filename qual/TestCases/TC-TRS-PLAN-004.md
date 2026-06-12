---
id: TC-TRS-PLAN-004
type: TestCase
testLevel: L3
status: draft
name: "Verify TestPlan demonstrated goals: E603 and the evidence-gap W614."
verifies:
  - REQ-TRS-PLAN-004
---

```gherkin
Feature: TestPlan demonstrated goals
  Scenario: unresolvable demonstrates target
    Given a TestPlan demonstrating a non-existent requirement
    When validate runs
    Then E603 is raised

  Scenario: demonstration gap
    Given an approved plan demonstrating a requirement that no member TestCase verifies
    When validate runs
    Then W614 is raised

  Scenario: demonstrated and covered
    Given an approved plan demonstrating a requirement a member TestCase verifies
    When validate runs
    Then neither E603 nor W614 is raised
```
