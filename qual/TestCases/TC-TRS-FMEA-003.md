---
id: TC-TRS-FMEA-003
type: TestCase
testLevel: L3
status: draft
name: "Verify fmeaRef on FaultTreeEvent and ftaRef on FMEAEntry cross-references raise W926/W927 on broken refs"
verifies:
  - REQ-TRS-FMEA-003
---

```gherkin
Feature: FMEA-FaultTree cross-reference fields fmeaRef and ftaRef

  Scenario: fmeaRef resolves cleanly when target FM-* exists
    Given a FaultTreeEvent with fmeaRef pointing to an existing FMEAEntry id FM-KERN-001
    When the tool validates the model
    Then no W926 is raised

  Scenario: fmeaRef raises W926 when target FM-* does not exist
    Given a FaultTreeEvent with fmeaRef pointing to FM-NONEXIST-001 which is absent from the model
    When the tool validates
    Then W926 is raised naming FM-NONEXIST-001

  Scenario: ftaRef resolves cleanly when target FTE-* exists
    Given an FMEAEntry row with ftaRef pointing to an existing FaultTreeEvent id FTE-KERN-001
    When the tool validates
    Then no W927 is raised

  Scenario: ftaRef raises W927 when target FTE-* does not exist
    Given an FMEAEntry row with ftaRef pointing to FTE-NONEXIST-001 which is absent from the model
    When the tool validates
    Then W927 is raised naming FTE-NONEXIST-001

  Scenario: refs command lists reverse fmeaRef link
    Given a model containing FaultTreeEvent FTE-KERN-001 with fmeaRef FM-KERN-001
    When the user runs refs FM-KERN-001
    Then the output lists FTE-KERN-001 as a referencing element via fmeaRef
```
