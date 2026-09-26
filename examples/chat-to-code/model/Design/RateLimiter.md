---
type: PartDef
features:
- name: buckets
  type: Part
  typedBy: Design::TokenBucket
- name: clock
  type: Part
  typedBy: Design::Clock
name: RateLimiter
satisfies:
- REQ-RL-004
- REQ-RL-005
- REQ-RL-006
- REQ-RL-007
---

Public entry point of the library (API: ADR-RL-003).

- Keeps one `TokenBucket` per key (0..*) and uses one `Clock`.
- Offers the two calling styles: `try_acquire(key) -> Decision` (refuse with retry-after) and `acquire(key)` (blocking).
- Is responsible for thread-safety of access to the buckets, using one global lock (ADR-RL-004).
