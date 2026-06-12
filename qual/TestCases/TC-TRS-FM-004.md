---
id: TC-TRS-FM-004
type: TestCase
testLevel: L3
status: draft
name: "Verify `mandatory:` membership field: a mandatory feature is core in deep analysis; legacy groupKind: mandatory still forced."
verifies:
  - REQ-TRS-FM-004
---

```gherkin
Feature: mandatory membership field
  Scenario: mandatory feature is core
    Given a parent FeatureDef with mandatory: true and groupKind: alternative and 2 children
    And a Configuration selecting one child
    When the tool runs `feature-check --deep`
    Then the model is sound (no E223 void, no E225 invalid-config)
    And the parent appears on the "core features:" line
  Scenario: legacy mandatory child still forced
    Given a child FeatureDef with legacy groupKind: mandatory under a parent
    When the tool runs `feature-check --deep`
    Then the child is treated as a forced/core feature
```
