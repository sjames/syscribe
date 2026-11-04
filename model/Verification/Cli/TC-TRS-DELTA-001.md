---
type: TestCase
id: TC-TRS-DELTA-001
name: "modelChanged reports added, changed and removed, debounced"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_model_changed.rs
verifies:
  - REQ-TRS-DELTA-001
tags:
  - lsp
---

```gherkin
Feature: model change notification

  Scenario: added, changed and removed
    Then a new, edited and deleted file are reported by qualified name with a rising sequence and a schema-valid payload

  Scenario: debouncing
    Then a burst is one notification and an edit with its undo is none

  Scenario: feature model and configurations
    Then each is flagged only when it changed

  Scenario: opt-in
    Then a client that did not opt in receives nothing and capabilities advertise the notification
```
