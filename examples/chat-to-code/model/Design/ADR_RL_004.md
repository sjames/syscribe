---
type: ADR
id: ADR-RL-004
name: Guard all limiter state with one global lock
status: accepted
---

## Context

REQ-RL-007 requires that concurrent callers never exceed what a key's bucket allows. ADR-RL-003 left locking granularity open: one lock for everything, or one lock per key.

## Decision

Use **one global lock** (a `threading.Lock` owned by the `RateLimiter`). It guards the key-to-bucket registry and every bucket's state.

- The lock is held only for the short check-and-update of a bucket (refill arithmetic, token decrement, retry-after computation).
- `acquire` never sleeps while holding the lock: it releases the lock, calls `clock.sleep(retry_after)`, then tries again.

## Alternatives considered

- **One lock per key**: more concurrency between different keys, but needs a second lock to protect the registry and more code to get right. Not justified for a library of a few hundred lines used inside a single process.

## Consequences

- Simple to reason about and to test (TC-RL-004).
- All keys contend on the same lock. The critical section is a few arithmetic operations, so this is expected to be negligible; revisit with a new ADR if measurement shows contention.
- Resolves the locking-granularity item left open in ADR-RL-003.

