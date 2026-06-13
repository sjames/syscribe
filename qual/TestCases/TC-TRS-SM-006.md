---
id: TC-TRS-SM-006
type: TestCase
testLevel: L3
status: draft
name: "Verify W079 (unresolved entry/do/exit/effect behavior reference) fires on a dangling effect and is silent when resolvable; decision transitions (guarded same-source branches) do not raise W072; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SM-008
---

Verify state-machine behavior-reference resolution and the decision-transition guarantee.

```gherkin
Feature: Behavior reference resolution (W079) and decision transitions

  Scenario: W079 — transition effect references a non-existent action
    Given a transition whose effect names an action that resolves to no element
    When the tool validates the model
    Then a W079 finding is emitted

  Scenario: no W079 — entry action and effect resolve
    Given a state machine whose entryAction and transition effect reference an existing action
    When the tool validates the model
    Then no W079 finding is emitted

  Scenario: decision transition does not raise W072
    Given two transitions from one source with the same accept payload, each carrying a guard
    When the tool validates the model
    Then no W072 finding is emitted

  Scenario: W079 draft-suppressed
    Given the dangling-effect machine with status draft
    When the tool validates the model
    Then no W079 finding is emitted

  Scenario: --deny W079 promotes to a gate failure
    Given the dangling-effect machine
    When the tool validates the model with --deny W079
    Then the tool exits non-zero
```
