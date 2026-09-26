---
type: Requirement
derivedFrom:
- REQ-RL-003
id: REQ-RL-008
name: The library shall be synchronous, Python 3.10+, standard library only
reqClass: system
reqDomain: software
status: verified
verificationMethod: inspection
breakdownAdr: ADR-RL-002
---

The library shall provide a synchronous API, shall support Python 3.10 and later, and shall import only from the Python standard library.

## Rationale

Matches the target service environment (REQ-RL-003).

