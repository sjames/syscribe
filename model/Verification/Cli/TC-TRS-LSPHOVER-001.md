---
type: TestCase
id: TC-TRS-LSPHOVER-001
name: "Hover and lenses report activity, parameter facts and who bound each parameter"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_ple_hover.rs
verifies:
  - REQ-TRS-LSPHOVER-001
tags:
  - lsp
---

```gherkin
Feature: variability hover and lenses

  Scenario: appliesWhen and bindings
    Then hover lists the Configurations an element is active in, and a binding or narrowing key shows range, unit, window and its constraints with their state

  Scenario: lenses
    Then a gated element and each parameter get their lens, an unbound parameter says so with its default, and the earlier lenses remain

  Scenario: responsiveness
    Then a lens request on the largest example model returns within a few seconds
```
