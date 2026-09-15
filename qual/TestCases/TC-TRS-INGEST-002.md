---
id: TC-TRS-INGEST-002
type: TestCase
testLevel: L3
status: draft
name: "Verify trace/matrix/safety-case/testplan roll up session-log scenario verdicts for a TestCase with no testFunctions, distinguishing prose-only evidence from an ingested pass, and never disturbing testFunctions-scored TestCases."
verifies:
  - REQ-TRS-INGEST-002
---

```gherkin
Feature: executed-evidence rollup from session-log
  Scenario: a TestCase with no testFunctions is unannotated before any ingest
    Given a TestCase with two Gherkin scenarios and no testFunctions, verifying a Requirement
    When "trace <req-id>" is run with no results sidecar present
    Then the TestCase is listed with no pass/fail annotation

  Scenario: every scenario recorded pass annotates the TestCase pass
    Given the same TestCase, with both its scenarios ingested as session-log passes
    When "trace <req-id>" is run
    Then the TestCase is annotated [pass]

  Scenario: any scenario recorded fail annotates the whole TestCase fail
    Given the same TestCase, with one scenario ingested pass and the other fail
    When "trace <req-id>" is run
    Then the TestCase is annotated [fail]

  Scenario: partial scenario coverage does not falsely read as pass
    Given the same TestCase, with only one of its two scenarios ingested (as pass)
    When "trace <req-id>" is run
    Then the TestCase is not annotated [pass] (and not [fail])

  Scenario: a TestCase with testFunctions ignores any session-log data
    Given a TestCase with testFunctions: already scored by cargo-json/junit results
    When session-log data unrelated to those functions is also ingested
    Then the TestCase's verdict is unchanged, sourced only from its testFunctions
```
