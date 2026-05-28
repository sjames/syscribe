---
type: PartDef
name: Diagnostic Security Layer
domain: software
satisfies:
  - REQ-ENG-SEC-002
  - REQ-ENG-SEC-004
features:
  - name: securityLevel
    type: ScalarValues::Integer
  - name: seedLength
    type: ScalarValues::Integer
    unit: bits
  - name: lockoutDurationMin
    type: ScalarValues::Integer
    unit: min
---

UDS (ISO 14229) security access layer managing authentication of diagnostic
and programming sessions. Implements challenge-response (seed-and-key) with
a minimum 128-bit security level before granting ECU programming access.

After three consecutive failed authentication attempts, the layer imposes a
10-minute lockout of the programming security level to resist brute-force attacks
(implements SC-ENG-002).
