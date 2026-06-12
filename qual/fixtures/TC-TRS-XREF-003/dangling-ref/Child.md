---
id: TC-XRF-DNG-001
type: TestCase
name: Fixture TC with dangling verifies reference
status: draft
testLevel: L3
verifies:
  - REQ-DANGLING-NONEXISTENT-999
---

```gherkin
Feature: Dangling reference

  Scenario: Baseline
    Given the reference cannot be resolved
    When validation runs
    Then E102 is emitted
```

