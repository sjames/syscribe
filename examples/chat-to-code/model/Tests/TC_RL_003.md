---
type: TestCase
id: TC-RL-003
name: Verify blocking acquire waits for the retry-after
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_acquire_does_not_sleep_when_token_available
  scenario: Blocking call with a token available does not wait
- function: test_acquire_sleeps_once_for_retry_after
  scenario: Blocking call sleeps once for the retry-after
- function: test_acquire_consumes_the_token_it_waited_for
  scenario: Blocking call consumes the token it waited for
testLevel: L1
verifies:
- REQ-RL-006
---

## Test Procedure

Uses a fake clock whose `sleep(s)` records `s` and advances `now()` by `s`, so no real time passes. Limiter configured with rate=2 and capacity=1.

```gherkin
Feature: Blocking acquire

  Scenario: Blocking call with a token available does not wait
    Given a limiter with rate 2 and capacity 1
    When key "a" calls acquire
    Then the clock recorded no sleeps

  Scenario: Blocking call sleeps once for the retry-after
    Given a limiter with rate 2 and capacity 1
    And key "a" has used its token
    When key "a" calls acquire
    Then the clock recorded exactly one sleep of 0.5 seconds

  Scenario: Blocking call consumes the token it waited for
    Given a limiter with rate 2 and capacity 1
    And key "a" has used its token
    When key "a" calls acquire
    And key "a" calls try_acquire
    Then the try_acquire call is refused
    And retry_after is 0.5
```

