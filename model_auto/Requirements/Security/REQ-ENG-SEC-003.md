---
type: Requirement
id: REQ-ENG-SEC-003
name: ECU firmware update shall require valid ECDSA P-256 signature and monotonic version counter
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-ENG-SYS-000
breakdownAdr: ADR-ENG-SYS-001
derivedFromSecurityGoal: CSG-ENG-003
---

The Engine ECU bootloader **shall** verify an ECDSA P-256 signature over the
SHA-256 hash of the firmware image before accepting any flash programming
request. The signature **shall** be verified against the OEM root CA certificate
chain stored in One-Time Programmable (OTP) memory.

The bootloader **shall** maintain a monotonically increasing 32-bit version
counter in OTP memory. Any firmware image presenting a version number lower
than the value stored in the OTP counter **shall** be rejected; the bootloader
**shall** return UDS Negative Response Code 0x22
(conditionsNotCorrect) and **shall** set a security diagnostic trouble code
(DTC) within 200 ms of the rejection.

The OTP version counter **shall** be incremented only after a firmware image
with a strictly higher version number has been successfully verified and
programmed, and only upon completion of the first successful post-update boot.

**Key compromise recovery**: a second X.509 certificate chain issued by a
dedicated OEM Recovery CA (held under physical OEM custody in a Hardware
Security Module) **shall** be available to perform a one-time OTP counter
reset. Counter reset operations **shall** require physical presence at an OEM
facility, **shall** be logged in a tamper-evident secure element, and **shall**
not be executable remotely or via the OBD-II port.
