---
type: TestCase
id: TC-RL-001
name: Verify per-key token bucket admission and refill
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_admits_up_to_capacity_then_refuses
  scenario: Calls are admitted until the bucket is empty
- function: test_keys_are_independent
  scenario: Keys are limited independently
- function: test_refills_with_elapsed_time_up_to_capacity
  scenario: Tokens refill with elapsed time up to capacity
testLevel: L1
verifies:
- REQ-RL-004
---

## Test Procedure

Uses a fake clock. Limiter configured with rate=1 token/s and capacity=3.

```gherkin
Feature: Per-key token bucket

  Scenario: Calls are admitted until the bucket is empty
    Given a limiter with rate 1 and capacity 3
    When key "a" calls try_acquire 4 times without time passing
    Then the first 3 calls are allowed
    And the 4th call is refused

  Scenario: Keys are limited independently
    Given a limiter with rate 1 and capacity 3
    And key "a" has used all its tokens
    When key "b" calls try_acquire
    Then the call is allowed

  Scenario: Tokens refill with elapsed time up to capacity
    Given a limiter with rate 1 and capacity 3
    And key "a" has used all its tokens
    When the clock advances 2 seconds
    Then key "a" is allowed exactly 2 calls and then refused
    When the clock advances 100 seconds
    Then key "a" is allowed exactly 3 calls and then refused
```

