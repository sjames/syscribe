---
type: TestCase
id: TC-TRS-VRES-002
name: "verification-depth excludes failing and unrun tests and lists them"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/verification_depth_results.rs
verifies:
  - REQ-TRS-VRES-002
tags:
  - verification
---

```gherkin
Feature: verification-depth with results

  Scenario: failing test does not count
    Given a requirement with an L3 test that passes and an L4 test that fails
    Then the levels are L3 only, flag single, and the L4 test is listed as failing

  Scenario: unrun automated test does not count
    Given an L4 test whose function is missing from the results
    Then it is listed as notRun and not counted

  Scenario: no results
    Then both tests count, as before
```
