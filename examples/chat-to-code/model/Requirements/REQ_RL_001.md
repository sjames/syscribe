---
type: Requirement
id: REQ-RL-001
name: The library shall limit the rate of API calls per key
reqClass: stakeholder
reqDomain: system
status: verified
verificationMethod: test
---

The library shall limit the rate at which API calls are admitted, separately for each key (for example, per API client).

## Rationale

The service needs to smooth its own traffic and protect its backend from any single client.

