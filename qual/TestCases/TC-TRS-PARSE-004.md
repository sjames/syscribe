---
id: TC-TRS-PARSE-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that .sysmlignore patterns suppress file discovery."
verifies:
  - REQ-TRS-PARSE-004
---

Verify that .sysmlignore patterns suppress file discovery.

```gherkin
Feature: .sysmlignore exclusion

  Scenario: Files matching .sysmlignore pattern are excluded
    Given a model directory with a valid element at Draft/MyElem.md
    And a .sysmlignore file containing the pattern Draft/
    When the tool is invoked against the model root
    Then Draft/MyElem.md is not processed as a model element

  Scenario: Absence of .sysmlignore causes no error
    Given a model directory with no .sysmlignore file
    When the tool is invoked against the model root
    Then the tool completes without an error related to missing .sysmlignore
```
