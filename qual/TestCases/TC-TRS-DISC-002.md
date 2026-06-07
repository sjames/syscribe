---
id: TC-TRS-DISC-002
type: TestCase
testLevel: L3
status: draft
title: "Verify the `feature <qname>` card: gated elements, selecting configurations, parameters; errors on unknown feature."
verifies:
  - REQ-TRS-DISC-002
---

```gherkin
Feature: feature command — per-feature card
  Scenario: card for a feature
    Given a product-line model
    When the tool runs `feature Features::Engine::Electric`
    Then it exits 0 and prints a "# Feature:" card for that feature
    And it lists under "Gates" the elements gated on that feature
    And it shows "Selected in" and the configuration ids that select it
  Scenario: card shows parameters
    When the tool runs `feature Features::Battery`
    Then the parameter name appears
  Scenario: unknown feature errors
    When the tool runs `feature Not::A::Feature`
    Then it exits non-zero
```
