---
type: TestCase
id: TC-RL-002
name: Verify retry-after reported for refused calls
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_refused_call_reports_retry_after
  scenario: Refused call reports the time until a token is available
- function: test_retry_after_shrinks_as_time_passes
  scenario: Retry-after shrinks as time passes
- function: test_admitted_after_retry_after_elapses
  scenario: Call is admitted once the reported retry-after has elapsed
- function: test_admitted_call_has_zero_retry_after
  scenario: Admitted call reports zero retry-after
testLevel: L1
verifies:
- REQ-RL-005
---

## Test Procedure

Uses a fake clock. Limiter configured with rate=2 tokens/s and capacity=1, so one token takes 0.5 s to refill.

```gherkin
Feature: Retry-after on refusal

  Scenario: Refused call reports the time until a token is available
    Given a limiter with rate 2 and capacity 1
    And key "a" has used its token
    When key "a" calls try_acquire
    Then the call is refused
    And retry_after is 0.5

  Scenario: Retry-after shrinks as time passes
    Given a limiter with rate 2 and capacity 1
    And key "a" has used its token
    When the clock advances 0.25 seconds
    And key "a" calls try_acquire
    Then the call is refused
    And retry_after is 0.25

  Scenario: Call is admitted once the reported retry-after has elapsed
    Given a limiter with rate 2 and capacity 1
    And key "a" has used its token
    And a refused call reported a retry_after
    When the clock advances by that retry_after
    And key "a" calls try_acquire
    Then the call is allowed

  Scenario: Admitted call reports zero retry-after
    Given a limiter with rate 2 and capacity 1
    When key "a" calls try_acquire
    Then the call is allowed
    And retry_after is 0.0
```

