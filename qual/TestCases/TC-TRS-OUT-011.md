---
id: TC-TRS-OUT-011
type: TestCase
testLevel: L3
status: draft
name: "Verify the verification-depth report: per-requirement level depth, flags, filters, JSON, and --min-levels gate."
verifies:
  - REQ-TRS-OUT-011
---

Verify the verification-depth report's depth flags, filters, JSON, and gating.

```gherkin
Feature: verification-depth and independence report

  Scenario: a requirement verified only by one L5 test is flagged hil-only
    Given a SIL-4 requirement verified by a single active L5 TestCase
    When the tool runs `verification-depth`
    Then that requirement is shown with flag hil-only

  Scenario: a requirement with no active verification is flagged none
    Given a SIL-4 requirement with no active verifying TestCase
    When the tool runs `verification-depth`
    Then that requirement is shown with flag none

  Scenario: a requirement verified at two distinct levels is ok
    Given a requirement verified by an active L3 and an active L5 TestCase
    When the tool runs `verification-depth`
    Then that requirement is shown with flag ok

  Scenario: --json emits the per-requirement array
    When the tool runs `verification-depth --json`
    Then each object carries id, levels, count and flag

  Scenario: --min-levels gates insufficient depth
    Given SIL-4 requirements that are not all verified at two distinct levels
    When the tool runs `verification-depth --sil 4 --min-levels 2`
    Then the tool exits non-zero
```
