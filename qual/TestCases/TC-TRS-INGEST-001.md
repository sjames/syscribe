---
id: TC-TRS-INGEST-001
type: TestCase
testLevel: L3
status: draft
name: "Verify session-log ingestion parses valid records into the results sidecar and hard-fails on malformed/missing steps, unrecognized result, or an empty array, without clobbering an existing sidecar."
verifies:
  - REQ-TRS-INGEST-001
---

```gherkin
Feature: session-log ingestion
  Scenario: a well-formed session-log file ingests successfully
    Given a JSON array of valid session-log records for an existing TestCase's scenarios
    When "ingest-results --format session-log <file>" is run
    Then the sidecar is written with a verdict per (testCase, scenario) pair

  Scenario: a record with empty steps fails ingestion
    Given a session-log record whose steps is an empty array
    When ingestion is run
    Then the command exits non-zero, names the offending record, and any existing sidecar is unchanged

  Scenario: a record with a missing steps field fails ingestion
    Given a session-log record with no steps field at all
    When ingestion is run
    Then the command exits non-zero and names the offending record

  Scenario: a record with an unrecognized result value fails ingestion
    Given a session-log record whose result is not pass/fail/unknown
    When ingestion is run
    Then the command exits non-zero, naming the unrecognized value

  Scenario: an empty input array fails ingestion
    Given a session-log file containing an empty JSON array
    When ingestion is run
    Then the command exits non-zero rather than writing an empty result set

  Scenario: --format session-log is never inferred from the file extension
    Given a session-log JSON file
    When "ingest-results <file>" is run with no --format
    Then the format is not silently inferred as session-log
```
