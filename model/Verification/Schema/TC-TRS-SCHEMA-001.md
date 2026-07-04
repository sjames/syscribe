---
type: TestCase
id: TC-TRS-SCHEMA-001
name: "W047 fires on an unrecognized frontmatter key and is exempt for custom_fields"
status: draft
testLevel: L2
tags:
  - schema
  - validation
verifies:
  - REQ-TRS-SCHEMA-001
---

Verifies that an unrecognized top-level frontmatter key raises the advisory `W047`
warning naming the key and file, that a recognized field does not, and that the same
data placed under `custom_fields:` is exempt.

```gherkin
Feature: Unrecognized frontmatter fields warn

  Scenario: An unknown top-level key raises W047
    Given an element whose frontmatter declares the key "wibble: 3"
    When the model is validated
    Then a W047 warning is reported for that element
    And the message names the key "wibble" and points to custom_fields

  Scenario: A recognized field does not raise W047
    Given an element whose frontmatter declares only recognized fields
    When the model is validated
    Then no W047 warning is reported for that element

  Scenario: custom_fields keys are exempt
    Given an element that places "wibble: 3" under custom_fields
    When the model is validated
    Then no W047 warning is reported for that element

  Scenario: W047 is advisory but gateable
    Given an element with an unrecognized key
    When the model is validated without --deny
    Then validation does not fail on account of W047
    And validating with --deny W047 turns it into a failure
```
