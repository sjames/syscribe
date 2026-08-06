---
type: Requirement
id: REQ-DRONE-VERIFY-001
name: "Thrust margin shall be independently verifiable"
status: draft
reqDomain: software
tags:
  - propulsion
  - verification
---

The propulsion subsystem shall expose enough instrumentation that its thrust margin can be independently verified against `REQ-DRONE-THRUST-001` during integration testing.

## Rationale

Targeted by the SysML v2 `thrustCheck` requirement usage's own `verify 'REQ-DRONE-VERIFY-001';` statement — demonstrating `REQ-TRS-SYSMLV2-003`'s `verify` keyword (as opposed to `satisfy`, demonstrated on the two requirements above). Left `status: draft` deliberately: this requirement's only trace link in this demo is that SysML v2 `verify` (which does not carry a stable id, so it cannot populate the native `verifiedBy` reverse index — see `REQ-TRS-SYSMLV2-004`'s task report) — a real project would still add a closing native `TestCase` before promoting it to `approved`.
