---
id: TC-TRS-VAL-010
type: TestCase
testLevel: L3
status: draft
title: "Verify function-level traceability (W009) across all supported languages and generic files."
verifies:
  - REQ-TRS-VAL-010
---

Verify that `W009` is emitted when a `testFunctions[].function` does not resolve in an existing `sourceFile`, and is *not* emitted when it does — across Rust, Java, C, C++, Kotlin, shell, and generic test files.

```gherkin
Feature: Function-level traceability (W009)

  Scenario: Resolving functions across all languages produce no W009
    Given TestCases whose sourceFile defines the named test in Rust, Java, C, C++, Kotlin, shell, and a generic .robot file
    When the tool is invoked
    Then no W009 finding is emitted

  Scenario: A renamed source function produces W009
    Given a TestCase whose Rust sourceFile no longer defines the named function
    When the tool is invoked
    Then a W009 finding names that function

  Scenario: A missing test in a generic file produces W009
    Given a TestCase whose generic sourceFile no longer contains the named test
    When the tool is invoked
    Then a W009 finding names that test
```
