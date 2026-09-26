---
type: Requirement
derivedFrom:
- REQ-RL-003
id: REQ-RL-007
name: Concurrent calls shall never exceed what the bucket allows
reqClass: system
reqDomain: software
status: verified
verificationMethod: test
breakdownAdr: ADR-RL-002
---

When multiple threads call the library concurrently for the same key, the total admitted shall never exceed what that key's token bucket allows.

## Rationale

The service calls the limiter from multiple threads; a race must not let extra calls through.

