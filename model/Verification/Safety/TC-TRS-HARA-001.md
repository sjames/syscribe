---
type: TestCase
id: TC-TRS-HARA-001
name: "W800 skips QM hazardous events and still flags ASIL A-D events"
status: active
testLevel: L1
sourceFile: repo:crates/syscribe-model/src/validator.rs
verifies:
  - REQ-TRS-HARA-001
tags:
  - safety
---

```gherkin
Feature: W800 and QM hazardous events

  Scenario: QM event needs no goal
    Given a HazardousEvent S1/E4/C1 with no SafetyGoal
    Then W800 is not raised

  Scenario: ASIL event without goal
    Given a HazardousEvent S3/E4/C3 with no SafetyGoal
    Then W800 is raised

  Scenario: incomplete rating
    Given a HazardousEvent with only severity set and no SafetyGoal
    Then W800 is raised
```
