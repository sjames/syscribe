---
id: TC-TRS-SM-002
type: TestCase
testLevel: L3
status: draft
name: "Verify W075 — legacy from/to/trigger transition keys raise the deprecation warning while still contributing the correct edge; canonical keys are silent; draft-suppressed; gateable with --deny W075."
verifies:
  - REQ-TRS-SM-002
---

Verify the deprecation of the legacy `from`/`to`/`trigger` transition keys.

```gherkin
Feature: Legacy transition-key deprecation (W075)

  Scenario: W075 — legacy from/to/trigger keys raise the deprecation warning
    Given a StateDef whose transitions use from/to/trigger
    When the tool validates the model
    Then a W075 finding is emitted

  Scenario: no W075 — canonical source/target/accept keys are silent
    Given the same machine written with source/target/accept
    When the tool validates the model
    Then no W075 finding is emitted

  Scenario: legacy keys still contribute the correct edge
    Given a StateDef using from/to/trigger that forms a fully connected machine with one initial
    When the tool validates the model
    Then no dead-state (W070) or trap-state (W071) warning is emitted

  Scenario: W075 is draft-suppressed
    Given the legacy-key machine with the StateDef status set to draft
    When the tool validates the model
    Then no W075 finding is emitted

  Scenario: --deny W075 promotes the warning to a gate failure
    Given the legacy-key machine
    When the tool validates the model with --deny W075
    Then the tool exits non-zero
```
