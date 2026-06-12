---
type: VulnerabilityReport
id: VR-ENG-001
name: Unauthenticated CAN message injection via OBD-II port
status: mitigated
cvssScore: 7.1
attackVector: physical
mitigatedBy:
  - SC-ENG-001
---

## Summary

The Engine ECU accepts powertrain CAN frames without message authentication
when the SecOC software component (`System::Software::CANSecurityModule`) is
not active or has not yet initialised at startup. During the post-reset
initialisation window (approximately 150 ms), an attacker with physical OBD-II
access can inject forged torque-command frames.

## Impact

An injected torque command frame may cause unintended vehicle acceleration
before the safety monitor reaches its operational state. This aligns with
damage scenario `DS-ENG-001` in `Security::TARA-ENG-001`.

## Mitigation

Security control `SC-ENG-001` (AUTOSAR SecOC CMAC-AES-128) prevents injection
under normal operation. The startup window is addressed by holding the
throttle actuator enable line low until SecOC is fully initialised, enforced
by the hardware watchdog reset output.

## References

- ISO/SAE 21434 §14 — Vulnerability management process
- Related threat: `TS-ENG-001` (replay via OBD-II port)
