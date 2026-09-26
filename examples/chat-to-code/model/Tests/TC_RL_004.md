---
type: TestCase
id: TC-RL-004
name: Verify concurrent calls never exceed the bucket
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_concurrent_callers_never_exceed_capacity
  scenario: Concurrent callers are admitted no more than the capacity
testLevel: L2
verifies:
- REQ-RL-007
---

## Test Procedure

Uses real threads and a fake clock that never advances, so refill cannot add tokens during the run. A barrier releases all threads at once to maximise contention, and the interpreter's thread-switch interval is set very low. The scenario is repeated for 30 rounds on fresh limiters, because a missing lock only loses a token occasionally; against an unlocked implementation this detected the race in 30 of 30 runs (a single round did in 4 of 40).

```gherkin
Feature: Thread safety

  Scenario: Concurrent callers are admitted no more than the capacity
    Given a limiter with rate 1 and capacity 50
    And a frozen clock
    When 200 threads each call try_acquire once for key "a" at the same time
    Then exactly 50 calls are allowed
    And 150 calls are refused
```

