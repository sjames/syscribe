---
id: TC-TRS-MG-012
type: TestCase
testLevel: L3
status: draft
title: "Verify trade-study ambiguous binding handling: colliding final-segment bindings => n/a; an exact key wins; a single segment match still resolves."
verifies:
  - REQ-TRS-MG-012
---

```gherkin
Feature: trade-study treats an ambiguous parameterBindings match as unevaluable
  Scenario: two bindings sharing a final segment make the cell n/a
    Given a Configuration binding SubsysA.speed and SubsysB.speed and a MoE expression using bare speed
    When trade-study is run
    Then that MoE cell is reported n/a

  Scenario: an exact key match wins despite a colliding segment
    Given the same colliding bindings plus an exact speed binding
    When trade-study is run
    Then the cell resolves to the exact binding value and is not n/a

  Scenario: a single segment match still resolves
    Given a Configuration with a single SubsysA.speed binding and a MoE expression using bare speed
    When trade-study is run
    Then the cell resolves to that value and is not n/a
```
