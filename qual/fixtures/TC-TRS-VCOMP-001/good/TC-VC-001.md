---
type: TestCase
id: TC-VC-001
name: bench
status: active
testLevel: L3
subject: Exec::Bench
verifies: [REQ-VC-001, REQ-VC-002]
---

```gherkin
Feature: bench

  @invariant
  Scenario: a sample opens the relay
    When stim receives Go
    Then rel is in state Open
```
