---
type: ADR
id: ADR-RL-003
name: Minimal public API of the ratelimit module
status: accepted
---

## Context

The library is a single module, `src/ratelimit.py`, tested by pytest functions in `tests/test_ratelimit.py` using a fake clock. The tests need a concrete, small public API. ADR-RL-001 fixes the algorithm and ADR-RL-002 the environment; this ADR fixes only the API surface.

## Decision

The module exposes four names:

- `RateLimiter(rate, capacity, *, clock=None)`
  - `rate`: tokens added per second (sustained rate).
  - `capacity`: maximum tokens a bucket holds (burst size).
  - One `rate`/`capacity` pair applies to every key.
  - A key's bucket is created on first use and starts full.
  - `clock` defaults to `SystemClock()`.
- `RateLimiter.try_acquire(key) -> Decision`: non-blocking. If a token is available it is consumed and the call is admitted; otherwise nothing is consumed and the call is refused.
- `RateLimiter.acquire(key) -> None`: blocking. Returns once the call is admitted. While refused it calls `clock.sleep(retry_after)` and tries again.
- `Decision`: immutable result with `allowed: bool` and `retry_after: float`. `retry_after` is in seconds and is `0.0` when the call is allowed.
- `Clock`: protocol with `now() -> float` (monotonic seconds) and `sleep(seconds) -> None`. `SystemClock` implements it with `time.monotonic` and `time.sleep`. Tests supply a fake `Clock`.

Every call consumes exactly one token.

## Not decided

Deliberately left out to keep the API small; each can be added later by a new ADR:

- weighted call cost
- per-key rate/capacity
- a timeout for `acquire`
- idle-key eviction
- locking granularity (since decided in ADR-RL-004)
- validation and exception types for invalid `rate`/`capacity`

## Consequences

- The fake clock in tests only needs `now()` and `sleep()`, and `sleep()` can advance time so blocking calls are testable without real waiting.
- Callers read `Decision.allowed` and `Decision.retry_after`; there is no exception for refusal.

