---
type: TestCase
id: TC-TRS-JUNIT-001
name: "JUnit classname keys and flaky verdict"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/junit_fidelity.rs
verifies:
  - REQ-TRS-JUNIT-001
tags:
  - results
---

```gherkin
Feature: JUnit fidelity

  Scenario: same test name in two classes
    Given A.test_x passes and B.test_x fails
    Then a reference to A#test_x passes, B#test_x fails, and the bare leaf fails

  Scenario: flaky
    Given a testcase with a flakyFailure child and no failure
    Then its verdict is flaky, the TestCase is not passing, and W010 reports it

  Scenario: failure plus reruns
    Then the verdict is fail

  Scenario: ingest summary
    Then the flaky count is printed
```
