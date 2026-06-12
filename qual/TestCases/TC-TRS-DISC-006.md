---
id: TC-TRS-DISC-006
type: TestCase
testLevel: L3
status: draft
name: "Verify orphan-feature warning W024 in feature-check: exactly one, names the orphan, not in base validate, gateable."
verifies:
  - REQ-TRS-DISC-006
---

```gherkin
Feature: W024 orphan feature in feature-check
  Scenario: orphan feature flagged
    Given a feature referenced by no appliesWhen and selected by no Configuration
    And a feature referenced by an appliesWhen, and a feature selected by a Configuration
    When the tool runs `feature-check`
    Then exactly one W024 is emitted naming the orphan feature only
  Scenario: base validate never emits W024
    When the tool runs `validate`
    Then no W024 is emitted
  Scenario: W024 is gateable
    When the tool runs `feature-check --deny W024`
    Then it exits 2
```
