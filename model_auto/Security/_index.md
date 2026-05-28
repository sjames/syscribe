---
type: Package
name: Security
---

This package contains cybersecurity analysis artefacts for the Engine ECU, performed per
ISO/SAE 21434:2021. The analysis covers the CAN bus interface and the OBD-II diagnostic
port as the primary attack surfaces identified during the cybersecurity scope definition.

## Contents

| Element | Type | Description |
|---|---|---|
| `TARA-ENG-001` | TARASheet | Full TARA: 4 damage scenarios, 4 threat scenarios, 4 goals, 4 controls |
| `VR-ENG-001` | VulnerabilityReport | CAN replay attack during SecOC startup window — **closed** |
| `VR-ENG-002` | VulnerabilityReport | Firmware rollback via OBD-II — **open** (W803) |

## Attack surfaces in scope

**CAN bus** — The powertrain CAN bus is shared with the transmission and brake control modules.
An attacker with physical access to the OBD-II port can inject CAN frames. AUTOSAR SecOC
(SC-ENG-001) provides MAC authentication; the startup window vulnerability (VR-ENG-001) is
mitigated by actuator enable lockout during boot.

**OBD-II port** — The diagnostic port supports UDS programming sessions which could enable
firmware replacement or calibration corruption. SC-ENG-002 (seed-and-key), SC-ENG-003 (ECDSA
firmware signing), and SC-ENG-004 (memory read access control) provide defence in depth.

## Open risks

`VR-ENG-002` represents an open risk pending implementation of the monotonic OTP rollback
counter specified in `REQ-ENG-SEC-003`. Until closed, the residual risk is accepted by the
product security owner under a documented waiver with a target remediation milestone.
