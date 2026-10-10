---
type: TestCase
id: TC-TRS-DFA-001
name: "DependentFailureAnalysis validates and excuses W034"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/dfa.rs
verifies:
  - REQ-TRS-DFA-001
tags:
  - safety
---

```gherkin
Feature: dependent failure analysis

  Scenario: a valid analysis
    Then validate reports no E890, E891, E892 or W890

  Scenario: defects
    Then fewer than two analyses, an unresolved entry, a bad kind or coupling factor and an unmitigated approved resource are reported

  Scenario: W034 excusal
    Then an approved DFA naming both co-hosted elements silences W034 and a draft one does not
```
