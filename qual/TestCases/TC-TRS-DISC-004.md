---
id: TC-TRS-DISC-004
type: TestCase
testLevel: L3
status: draft
title: "Verify `list <type> --feature <F>`: filters to elements gated on F; errors on unknown feature."
verifies:
  - REQ-TRS-DISC-004
---

```gherkin
Feature: list --feature — filter by gating feature
  Scenario: filter to gated elements
    Given a product-line model
    When the tool runs `list PartDef --feature Features::Engine::Electric`
    Then it lists the element(s) gated on that feature
    And it does NOT list an always-active element of the same type
  Scenario: unknown feature errors
    When the tool runs `list PartDef --feature Not::A::Feature`
    Then it exits non-zero
```
