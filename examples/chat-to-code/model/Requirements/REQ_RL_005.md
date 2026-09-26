---
type: Requirement
breakdownAdr: ADR-RL-001
derivedFrom:
- REQ-RL-002
id: REQ-RL-005
name: A refused non-blocking call shall report a retry-after
reqClass: system
reqDomain: software
status: verified
verificationMethod: test
---

When a non-blocking acquire is refused, the library shall report the minimum duration after which the same call would be admitted, assuming no other call is made for that key in the meantime. The value shall be computed from the bucket's rate and current token count.

## Rationale

ADR-RL-001 makes retry-after exact, so callers can be told precisely when to retry.

