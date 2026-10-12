---
id: TC-TRS-MODELS-001
type: TestCase
testLevel: L3
status: draft
name: "Verify models: links are checked against the architecture: targets, ports, connections and TestCase subjects."
verifies:
  - REQ-TRS-MODELS-001
---

```gherkin
Feature: behavioural model trace
  Scenario: a consistent model is clean
    Given executable parts that model a software manager and relay, joined like the architecture
    When the tool runs `validate`
    Then none of E990, E991, W990, W991, W992 is reported
  Scenario: each defect is reported
    Given a models: typo, an extra executable port, a wrong-direction port, an uncovered architecture port, a missing executable connection and a requirement named in abstracts: that the TestCase verifies
    When the tool runs `validate`
    Then E990, E991, W990, W991 and W992 are reported
```
