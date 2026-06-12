---
id: TC-XRF-UNR-001
type: TestCase
name: Fixture TC with unresolvable verifies reference
status: draft
testLevel: L3
verifies:
  - REQ-XREF-ABSOLUTE-NONEXISTENT-999
---

```gherkin
Feature: Unresolvable absolute reference

  Scenario: Baseline
    Given the reference cannot be resolved
    When validation runs
    Then E102 is emitted
```

