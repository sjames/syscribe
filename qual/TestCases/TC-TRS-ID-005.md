---
id: TC-TRS-ID-005
type: TestCase
testLevel: L3
status: draft
title: "Verify configurable stable-ID suffix width: 3-8 default, E023 over the cap, E006 under 3, configurable via [ids] max_digits."
verifies:
  - REQ-TRS-ID-005
---

```gherkin
Feature: configurable stable-ID suffix width
  Scenario: default cap of 8
    Given a model with 3-, 8-, 9- and 2-digit suffix ids and no [ids] config
    When validate runs
    Then the 3- and 8-digit ids are clean
    And the 9-digit id raises E023 naming it and the cap
    And the 2-digit id raises E006 (minimum is 3)
    And a TestCase verifying the 9-digit id still resolves (no E102)

  Scenario: configurable cap
    Given [ids] max_digits = 9
    When validate runs
    Then a 9-digit id is clean
    Given [ids] max_digits = 4
    When validate runs
    Then a 5-digit id raises E023
```
