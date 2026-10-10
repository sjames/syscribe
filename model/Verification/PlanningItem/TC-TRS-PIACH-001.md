---
type: TestCase
id: TC-TRS-PIACH-001
name: "achieves accepts goals, ADRs, arguments, plans and baselines with a completion check"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/planning_achieves_targets.rs
verifies:
  - REQ-TRS-PIACH-001
tags:
  - planning
---

```gherkin
Feature: PlanningItem achieves targets

  Scenario: accepted targets
    Then achieves naming a SafetyGoal, CybersecurityGoal, ADR, Argument, TestPlan or Baseline raises no E715

  Scenario: rejected targets
    Then achieves naming a PartDef raises E715 listing the accepted types

  Scenario: completion
    Then a done item achieving a draft ADR raises W315 and an accepted ADR does not

  Scenario: set
    Then set achieves.add accepts an ADR and still refuses a PartDef
```
