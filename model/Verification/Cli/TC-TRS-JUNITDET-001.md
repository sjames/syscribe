---
type: TestCase
id: TC-TRS-JUNITDET-001
name: "junit failure details are retained and missing vs skipped functions are summarised"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/junit_details.rs
verifies:
  - REQ-TRS-JUNITDET-001
tags:
  - cli
---

```gherkin
Feature: junit details

  Scenario: details
    Then message and time of a failing, skipped and flaky case are kept and a passing case adds none

  Scenario: failures listing
    Then results failures prints the function, verdict, message and time

  Scenario: missing vs skipped summary
    Then ingest prints how many expected functions are missing and how many were skipped
```
