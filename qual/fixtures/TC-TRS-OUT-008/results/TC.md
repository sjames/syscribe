---
id: TC-ING-001
type: TestCase
title: "Mutex acquire/release"
status: active
testLevel: L3
verifies:
  - REQ-ING-001
testFunctions:
  - function: "mutex::tests::acquire_ok"
    scenario: "Acquire succeeds"
  - function: "mutex::tests::release_fails"
    scenario: "Release fails"
  - function: "mutex::tests::never_ran"
    scenario: "Never ran"
---

```gherkin
Feature: Mutex
  Scenario: Acquire succeeds
    Given a free mutex
    When acquired
    Then ok
  Scenario: Release fails
    Given a held mutex
    When released
    Then ok
  Scenario: Never ran
    Given a mutex
    When nothing
    Then nothing
```
