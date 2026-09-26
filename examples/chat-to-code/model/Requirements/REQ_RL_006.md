---
type: Requirement
breakdownAdr: ADR-RL-001
derivedFrom:
- REQ-RL-002
id: REQ-RL-006
name: A blocking call shall wait for the computed retry-after
reqClass: system
reqDomain: software
status: verified
verificationMethod: test
---

When a blocking acquire cannot be admitted immediately, the library shall wait for the retry-after duration and then admit the call, without busy-polling.

## Rationale

The exact retry-after from ADR-RL-001 lets the library sleep once instead of polling.

