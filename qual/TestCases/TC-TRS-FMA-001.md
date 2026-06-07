---
id: TC-TRS-FMA-001
type: TestCase
testLevel: L3
status: draft
title: "Verify the Boolean encoding via solver-observable semantics."
verifies:
  - REQ-TRS-FMA-001
---

```gherkin
Feature: Feature model Boolean encoding
  Scenario: mandatory feature is core, alternative rejects two, optional is free
    Given encoded feature models
    When the tool runs feature-check --deep
    Then a mandatory feature is reported core
    And selecting two alternatives is an invalid configuration
    And a plain optional feature is not core
    And a contradictory model is void
```
