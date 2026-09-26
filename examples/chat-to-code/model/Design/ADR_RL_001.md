---
type: ADR
id: ADR-RL-001
name: Use a token bucket for rate limiting
status: accepted
---

## Context

The library rate-limits API calls per key (e.g. per API client) inside a single-process service, called from multiple threads. Its purpose is to smooth our own traffic and protect our own backend; there is no downstream API enforcing a strict "N calls per window" that we must never exceed.

Callers must be able to either try and be rejected with a retry-after, or block until allowed. Candidate algorithms: fixed window, sliding window (log or counter), token bucket.

## Decision

Use a **token bucket** per key, parameterised by a sustained rate and a burst capacity.

## Alternatives considered

- **Fixed window**: simplest, but allows up to 2N calls across a window boundary, and blocked callers all wake at the boundary.
- **Sliding window log**: strict "never more than N in any T", but O(N) memory per key. The strictness is not needed here.
- **Sliding window counter**: approximate, with no advantage over token bucket for this use.

## Consequences

- Retry-after is exact: (cost - tokens) / rate. The same value serves the reject path and the blocking path.
- Per-key state is small (token count and last-refill timestamp).
- Users configure two parameters (rate and capacity), so "N per T" must be mapped onto them.
- A full bucket can emit up to about 2N calls in some window of length T. This is accepted because the limiter smooths our own traffic rather than enforcing a strict window.

