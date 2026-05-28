---
type: Part
name: CAN Security Module
typedBy: System::Software::CANSecurityModule
domain: software
---

CANSecurityModule AUTOSAR SWC instance, event-driven (activated by the CanIf pre-transmit
hook and post-receive hook). Implements AUTOSAR SecOC (Secure Onboard Communication) for
MAC generation on outbound safety-critical CAN PDUs and MAC verification on inbound PDUs.

## Cryptographic configuration

- Algorithm: CMAC-AES-128 (AUTOSAR Csm job CsmMacGenerate/CsmMacVerify)
- Key length: 128 bits, stored in the ECU HSM (Hardware Security Module) key store
- MAC length: 24 bits (3 bytes), appended to the last three bytes of each protected frame
- Freshness counter: 4-bit truncated counter per PDU, transmitted in the frame
- Key lifecycle: provisioned at ECU programming time; re-keying requires security access 0x27

## Protected PDUs

The following PDUs carry SecOC MACs (SC-ENG-001):
- Torque command frame (0x0C8, from ThrottleControl, 10 ms)
- Transmission shift request (0x0C9, 20 ms)
- Brake torque coordination request (0x0CA, 10 ms)

A frame received with an invalid MAC is discarded; three consecutive failures on any PDU
set DTC U0001 (lost communication) and flag a security event to the DiagnosticSecurityLayer
audit log.

## Startup window

During the first 200 ms after reset (SecOC key load not yet complete), inbound MACs cannot
be verified. The actuator enable line is held low (throttle disabled) during this window to
mitigate VR-ENG-001. Once keys are loaded, the startup lockout is released.
