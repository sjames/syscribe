---
type: PartDef
name: TokenBucket
satisfies:
- REQ-RL-004
- REQ-RL-005
---

Rate-limit state for a single key: token count and last-refill timestamp, configured by a sustained rate and a burst capacity (see ADR-RL-001).

- Refills from elapsed time when consulted.
- Decides whether a request is allowed and, if not, computes the retry-after.
- Knows nothing about keys, threads or how time is obtained.

