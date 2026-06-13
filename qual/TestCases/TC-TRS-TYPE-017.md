---
id: TC-TRS-TYPE-017
type: TestCase
name: "Asset type validates with E861/E862/E863/E864/W810 rules"
status: active
testLevel: L1
verifies: [REQ-TRS-TYPE-017]
---

Verify Asset element validation: required fields (E861), id pattern (E862), cybersecurityProperties enum (E863), DamageScenario.assets type check (E864), and unreferenced Asset warning (W810).

```gherkin
Feature: Asset element validation

  Scenario: a well-formed Asset validates clean
    Given an Asset with id, name, status and a valid cybersecurityProperties entry
    And it is referenced by a DamageScenario.assets list
    When I validate the model
    Then no E861, E862, E863, E864 or W810 finding is reported

  Scenario: missing required fields and bad values are rejected
    Given an Asset missing id/name/status, with a non-ASSET id and an invalid cybersecurityProperties value
    When I validate the model
    Then the output contains "E861"
    And the output contains "E862"
    And the output contains "E863"

  Scenario: a DamageScenario.assets entry must resolve to an Asset
    Given a DamageScenario whose assets list references a non-Asset element
    When I validate the model
    Then the output contains "E864"

  Scenario: an unreferenced Asset warns
    Given an Asset not referenced by any DamageScenario.assets
    When I validate the model
    Then the output contains "W810"
```
