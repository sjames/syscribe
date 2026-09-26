---
type: Requirement
breakdownAdr: ADR-RL-001
derivedFrom:
- REQ-RL-001
id: REQ-RL-004
name: The library shall keep an independent token bucket per key
reqClass: system
reqDomain: software
status: verified
verificationMethod: test
---

The library shall maintain a separate token bucket, configured by a sustained rate and a burst capacity, for each key, and shall admit a call only if that key's bucket holds enough tokens for it. Consumption on one key shall not affect any other key.

## Rationale

Direct consequence of ADR-RL-001 (token bucket) applied per key.

