---
id: TC-TRS-VAL-014
type: TestCase
testLevel: L3
status: draft
name: "Verify W004/W009 fire for active TestCases only, while non-TestCase sourceFiles are still checked."
verifies:
  - REQ-TRS-VAL-014
---

Verify that source-drift checks are scoped to `active` TestCases: a `draft` TestCase with a missing `sourceFile` or unresolved `testFunctions` produces no `W004`/`W009`, an `active` one still does, and a non-`TestCase` element with a missing `sourceFile` is still flagged.

```gherkin
Feature: Drift checks scoped to active TestCases

  Scenario: Draft TestCase drift is suppressed
    Given a draft TestCase with a missing sourceFile and an unresolved testFunction
    When the tool is invoked
    Then no W004 or W009 is emitted for it

  Scenario: Active TestCase drift is reported
    Given an active TestCase with a missing sourceFile, and another with an unresolved testFunction
    When the tool is invoked
    Then W004 is emitted for the missing sourceFile and W009 for the unresolved testFunction

  Scenario: Non-TestCase sourceFiles are still checked
    Given a PartDef with a missing sourceFile
    When the tool is invoked
    Then W004 is emitted regardless of any status
```
