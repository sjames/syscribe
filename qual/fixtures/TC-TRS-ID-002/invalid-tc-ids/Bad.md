---
id: tc-001
type: TestCase
name: Invalid TC id (lowercase)
status: draft
testLevel: L3
verifies:
  - REQ-TST-XRF-001
---

Test case with an invalid id `tc-001` — lowercase, which the `TC-*` pattern never accepts
regardless of segment count. (A bare `TC-002`, missing the once-required category segment, is
valid since `REQ-TRS-ID-002` widened the pattern — see `TC-002.md` in `valid-tc-ids/` for that
positive case; this fixture exists specifically to keep a genuinely-invalid id covered.)

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```
