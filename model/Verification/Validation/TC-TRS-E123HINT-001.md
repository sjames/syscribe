---
type: TestCase
id: TC-TRS-E123HINT-001
name: "E123 messages carry the remedy"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/e123_hint.rs
verifies:
  - REQ-TRS-E123HINT-001
tags:
  - validation
---

```gherkin
Feature: E123 remedy

  Scenario: attribute typed by an item def (inline feature)
    Then the message suggests an attribute def and mentions type: Item only for flowing items

  Scenario: standalone usage
    Then the same remedy appears

  Scenario: item typed by an attribute def
    Then the mirror remedy appears
```
