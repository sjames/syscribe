---
id: TC-TRS-OUT-007
type: TestCase
testLevel: L3
status: draft
title: "Verify the structured model graph export (JSON + NDJSON, schemaVersion, resolved relationships)."
verifies:
  - REQ-TRS-OUT-007
---

Verify that `export` emits a versioned JSON document whose elements carry typed frontmatter and resolved reverse-index relationships, and that an NDJSON variant is available.

```gherkin
Feature: Structured model graph export

  Scenario: Export emits a versioned JSON document
    Given a model with a requirement verified by a test case
    When export is invoked
    Then the output is valid JSON carrying schemaVersion and an elements array

  Scenario: Requirements expose resolved verifiedBy
    Given a requirement covered by an active test case
    When export is invoked
    Then the requirement element lists that test case under computed.verifiedBy

  Scenario: TestCases round-trip their verifies list
    Given a test case that verifies a requirement
    When export is invoked
    Then the test case element carries that requirement in its verifies field

  Scenario: NDJSON variant emits a header then one element per line
    Given any model
    When export --ndjson is invoked
    Then the first line is a header carrying schemaVersion and elementCount
```
