---
type: Requirement
id: REQ-RL-003
name: The library shall be usable in the target service environment
reqClass: stakeholder
reqDomain: system
status: verified
verificationMethod: analysis
---

The library shall be usable from multiple threads of a single-process, synchronous Python service on Python 3.10 or later, without requiring third-party packages.

## Rationale

This is the environment the service runs in; the library must not add dependencies or an operational footprint beyond it.

