---
id: TC-TRS-OUT-014
type: TestCase
testLevel: L3
status: draft
name: "Verify list TestCase emits test-execution table and JSON with testFunctions; config+tag combined."
verifies:
  - REQ-TRS-OUT-014
  - REQ-TRS-TAG-002
---

```gherkin
Feature: list TestCase specialized output

  Scenario: human-readable table has execution-oriented columns
    Given a model with TestCase elements carrying id, name, testLevel, status, verifies, and tags
    When list TestCase runs
    Then the table has columns ID, Name, Level, Status, Verifies, Tags
    And the ID column shows the TC-* stable identifier
    And the Verifies column lists the requirement ids

  Scenario: JSON output includes testFunctions and sourceFile
    Given a TestCase with testFunctions and sourceFile declared
    When list TestCase --json runs
    Then each item carries testLevel, verifies, tags, sourceFile, and testFunctions
    And testFunctions contains the declared function entries

  Scenario: --config projection combined with --tag
    Given a model with a Configuration and TestCases some of which are appliesWhen-gated
    When list TestCase --config CONF-X --tag integration runs
    Then only TestCases active in CONF-X and carrying the integration tag are listed

  Scenario: non-TestCase types still use the generic table
    Given a model with Requirement elements
    When list Requirement runs
    Then the table has columns Qualified Name, Name / ID, Supertype / TypedBy, File
```
