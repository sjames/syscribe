---
type: Package
name: Security
---

Cybersecurity requirements are derived from the Threat Analysis and Risk Assessment (TARA)
in `Security/TARA-ENG-001`. Each requirement traces to a `CybersecurityGoal` via
`derivedFromSecurityGoal:` and carries `verificationMethod:` per W807.

## Requirements and CAL allocation

| ID | Cybersecurity goal | CAL | Control |
|---|---|---|---|
| `REQ-ENG-SEC-001` | `CSG-ENG-001` (CAN authenticity) | CAL 3 | AUTOSAR SecOC MAC |
| `REQ-ENG-SEC-002` | `CSG-ENG-002` (calibration integrity) | CAL 2 | UDS seed-and-key |
| `REQ-ENG-SEC-003` | `CSG-ENG-003` (firmware integrity) | CAL 3 | ECDSA P-256 signature |
| `REQ-ENG-SEC-004` | `CSG-ENG-004` (diagnostic confidentiality) | CAL 2 | UDS security access + audit log |

CAL (Cybersecurity Assurance Level) determines the rigour of the development and verification
activities required by ISO/SAE 21434 §10.4. CAL 3 requirements apply to functions directly
controlling actuators over CAN or protecting firmware integrity.

## Open vulnerability

`VR-ENG-002` (firmware rollback via OBD-II) is currently open (W803). Until the monotonic OTP
version counter in `REQ-ENG-SEC-003` is implemented and verified, firmware rollback remains a
residual risk. See `Security/VR-ENG-002.md` for the full risk assessment.
