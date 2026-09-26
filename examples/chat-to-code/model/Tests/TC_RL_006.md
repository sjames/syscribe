---
type: TestCase
id: TC-RL-006
name: Verify end-to-end limiting in a multi-threaded caller on the real clock
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_e2e_refused_key_gets_retry_after_and_other_key_unaffected
  scenario: A refused client gets a retry-after while another client is unaffected
- function: test_e2e_blocking_caller_waits_on_real_clock
  scenario: A blocking caller is held back on the real clock
- function: test_e2e_threads_are_held_to_the_sustained_rate
  scenario: Concurrent blocking threads are held to the sustained rate
testLevel: L3
verifies:
- REQ-RL-001
- REQ-RL-002
- REQ-RL-003
---

## Test Procedure

Uses the default `SystemClock` and real threads, with no fake clock, exactly as a service would use the library. Assertions are lower bounds on elapsed time (measured with `time.monotonic`), because sleeping can return late but not early; there are no upper bounds, so the test does not depend on machine speed.

```gherkin
Feature: End-to-end rate limiting

  Scenario: A refused client gets a retry-after while another client is unaffected
    Given a limiter with rate 1 and capacity 2 using the real clock
    And client "a" has used both of its tokens
    When client "a" calls try_acquire
    Then the call is refused with a retry_after greater than 0
    When client "b" calls try_acquire
    Then the call is allowed

  Scenario: A blocking caller is held back on the real clock
    Given a limiter with rate 20 and capacity 1 using the real clock
    And client "a" has used its token
    When client "a" calls acquire
    Then the call returns
    And at least 0.04 seconds of real time have elapsed

  Scenario: Concurrent blocking threads are held to the sustained rate
    Given a limiter with rate 200 and capacity 1 using the real clock
    When 8 threads each call acquire 10 times for client "a"
    Then all 80 calls complete
    And at least 0.35 seconds of real time have elapsed
```

The last lower bound is (80 - 1) / 200 = 0.395 s minus a margin for timer granularity.

