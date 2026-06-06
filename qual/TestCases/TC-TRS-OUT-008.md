---
id: TC-TRS-OUT-008
type: TestCase
testLevel: L3
status: draft
title: "Verify test-result ingestion and W010 for failing/missing tests (cargo-json + JUnit)."
verifies:
  - REQ-TRS-OUT-008
---

Verify that ingesting test results raises `W010` for an `active` TestCase whose mapped functions failed or did not run, leaves passing functions unflagged, and that the resulting model can be gated with `--deny W010`. Both cargo-json and JUnit XML are exercised.

```gherkin
Feature: Test result ingestion (W010)

  Scenario: No results means no W010
    Given an active TestCase with testFunctions and no ingested results
    When the tool is invoked
    Then no W010 finding is emitted

  Scenario: cargo-json results flag failing and missing functions
    Given a cargo-json report where one function passed and one failed
    When the results are ingested and the tool is invoked
    Then W010 is emitted for the failed function and the function absent from the run
    And no W010 is emitted for the passing function

  Scenario: Failing tests can gate CI
    Given an active TestCase with a failing ingested function in an otherwise clean model
    When validate is invoked with --deny W010
    Then the exit code is 2

  Scenario: JUnit XML results are also supported
    Given a JUnit report where one testcase has a failure
    When the results are supplied via validate --results
    Then W010 is emitted for the failing and missing functions
```
