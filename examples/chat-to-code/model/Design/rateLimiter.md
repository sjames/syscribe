---
type: Part
multiplicity: '1'
name: rateLimiter
typedBy: Design::RateLimiter
satisfies:
- REQ-RL-008
---

The library's rate limiter as used by a service: one instance, shared by all calling threads.
