---
type: Part
name: Diagnostic Security Layer
typedBy: System::Software::DiagnosticSecurityLayer
domain: software
---

DiagnosticSecurityLayer AUTOSAR SWC instance, invoked by the AUTOSAR Dcm (Diagnostic
Communication Manager) on receipt of UDS service requests over the OBD-II port.

## Security access levels

| Level | UDS subfunction | Purpose | Seed length |
|---|---|---|---|
| 0x27/0x28 | SecurityAccess | Calibration flash programming | 128 bits |
| 0x11/0x12 | SecurityAccess | Diagnostic memory read (`readMemoryByAddress`) | 128 bits |

Level 0x27/0x28 grants access to reprogram the calibration flash sector and update
configuration data. Level 0x11/0x12 grants read access to mapped memory regions for
live data capture (REQ-ENG-SEC-004).

## Lockout policy

After three consecutive failed security access attempts (invalid key response), the ECU
enters a 10-minute diagnostic lockout. During lockout, all `SecurityAccess` requests are
rejected with NRC 0x37 (requiredTimeDelayNotExpired). The lockout timer persists across
key-off/on cycles (stored in non-volatile RAM).

## Audit logging

Every successful `readMemoryByAddress` request in an authenticated session is appended to
a 64-entry ring buffer in non-volatile memory. Each entry is protected by a 32-bit CMAC
truncated from the AES-128 log-entry MAC (SC-ENG-004). The ring buffer is readable only
under security access level 0x11/0x12, preventing an attacker from erasing their tracks.
