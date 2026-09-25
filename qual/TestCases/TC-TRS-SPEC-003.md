---
id: TC-TRS-SPEC-003
type: TestCase
testLevel: L3
status: draft
name: "Verify explain_finding returns real explanations for every catalogued code, including three-column tables and previously missing codes."
verifies:
  - REQ-TRS-SPEC-003
---

Drives the MCP server over stdio and asks `explain_finding` about codes from three-column
(`Code | Severity | Condition`) tables, codes that were missing from the catalogue, and codes whose
catalogue rows were stale.

```gherkin
Feature: validation-code catalogue completeness (TC-TRS-SPEC-003)

  Scenario: three-column table rows explain the condition, not the severity
    When explain_finding is called for E600, W610, W041, W042, E317 and W045
    Then none of the explanations is a bare severity word

  Scenario: previously missing codes are explained
    When explain_finding is called for E108, E231, E520, W047, W090, W404, W563 and I010
    Then each returns an explanation rather than an error

  Scenario: stale W007 and W010 rows are corrected
    When explain_finding is called for W007 and W010
    Then W007 describes an unused definition and W010 describes ingested test results
```
