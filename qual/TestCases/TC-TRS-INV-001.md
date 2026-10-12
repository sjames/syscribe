---
id: TC-TRS-INV-001
type: TestCase
testLevel: L3
status: draft
name: "Verify named properties, degraded clauses and the temporal operators within and always."
verifies:
  - REQ-TRS-INV-001
---

```gherkin
Feature: richer invariants
  Scenario: within is reached and kept
    Given a machine that opens its outputs 2.2 s after one event and opens one at 1 s, closes it at 1.5 s after another
    When the tool runs `simulate`
    Then `within 2 s` fails, `within 2.5 s` passes, and the property reached at 1 s fails as no longer holding at 1.5 s
  Scenario: always holds at every settled instant
    When the tool runs `simulate`
    Then `always` fails on the violation at 1.5 s although the property holds again at the end
  Scenario: degraded clauses
    When the tool runs `simulate` and `inject`
    Then a welded output passes in its degraded clause, two welded outputs report no applicable degraded clause, and the campaign reports the welded sets as survived (degraded)
  Scenario: malformed clauses are E886
    Given an Except before any Where and a step naming an undeclared property
    When the tool runs `validate`
    Then both are E886
```
