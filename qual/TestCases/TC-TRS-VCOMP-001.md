---
id: TC-TRS-VCOMP-001
type: TestCase
testLevel: L3
status: draft
name: "Verify verification composites: environment parts, architecture elements used directly, and their reports."
verifies:
  - REQ-TRS-VCOMP-001
---

```gherkin
Feature: verification composite
  Scenario: a verification composite is clean and its reports separate the environment
    Given a bench with an environment stimulus, an executable monitor that models the manager, and the relay architecture element itself
    When the tool runs `validate`, `simulate --coverage` and `inject`
    Then none of E992, W991, W992, W993 is reported
    And the stimulus is listed as an environment part, outside the coverage aggregate
  Scenario: each defect is reported
    Given a misspelt role, a portMap on the environment usage, an environment stimulus that models a sensor whose requirement the TestCase verifies, and a missing architecture connection into the relay
    When the tool runs `validate`
    Then E992, W993, W992 and W991 are reported
```
