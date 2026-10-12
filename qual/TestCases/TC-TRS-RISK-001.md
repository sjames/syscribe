---
id: TC-TRS-RISK-001
type: TestCase
testLevel: L3
status: draft
name: "Verify recorded fault-injection campaigns and residual-risk decisions."
verifies:
  - REQ-TRS-RISK-001
---

```gherkin
Feature: campaign evidence and residual risk
  Scenario: a recorded campaign is listed and deterministic
    Given a composite with an invariant that two single faults defeat
    When the tool runs `inject --record` twice and `results campaigns`
    Then the campaign is listed with 2 defeated sets and both records have the same content apart from the timestamp
  Scenario: defeats are tied to decisions
    Given a ResidualRisk accepting one of the two defeated sets
    When the tool runs `validate`
    Then one W940 is reported, for the other set
    And changing the composite makes the decision's campaign hash stale (W942) and the record out of date (W944)
  Scenario: the safety case shows the campaign
    When the tool runs `safety-case`
    Then a solution node counts 3 sets, 1 survived, 2 defeated, 1 explained
```
