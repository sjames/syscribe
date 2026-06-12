---
type: TARASheet
id: TARA-ENG-001
name: TARA — Engine ECU CAN bus and OBD-II interface
status: approved

damageTable:
  - id: DS-ENG-001
    name: Unauthorised torque command causes unintended vehicle acceleration
    damageSeverity: severe
    impactCategories:
      - safety
      - operational

  - id: DS-ENG-002
    name: Malicious ECU calibration enables persistent engine faults
    damageSeverity: major
    impactCategories:
      - safety
      - operational

  - id: DS-ENG-003
    name: Firmware rollback exposes patched vulnerabilities — persistent remote exploit
    damageSeverity: major
    impactCategories:
      - safety
      - operational
      - financial

  - id: DS-ENG-004
    name: Diagnostic data exfiltration reveals proprietary calibration maps
    damageSeverity: moderate
    impactCategories:
      - financial
      - privacy

threatTable:
  - id: TS-ENG-001
    name: Attacker replays captured CAN torque-request frame via OBD-II port
    attackFeasibility: medium
    attackVector: local
    damageScenarios:
      - DS-ENG-001

  - id: TS-ENG-002
    name: Attacker injects malicious calibration via unauthenticated UDS session
    attackFeasibility: low
    attackVector: local
    damageScenarios:
      - DS-ENG-002

  - id: TS-ENG-003
    name: Attacker downloads unsigned legacy firmware image via OBD-II reflash
    attackFeasibility: low
    attackVector: local
    damageScenarios:
      - DS-ENG-003

  - id: TS-ENG-004
    name: Attacker uses valid but stolen OEM diagnostic tool to extract calibration data in UDS readMemoryByAddress session
    attackFeasibility: medium
    attackVector: local
    damageScenarios:
      - DS-ENG-004

goalTable:
  - id: CSG-ENG-001
    name: Ensure integrity and freshness of safety-critical CAN messages
    calLevel: CAL3
    securityProperty: integrity
    threatScenarios:
      - TS-ENG-001

  - id: CSG-ENG-002
    name: Ensure authenticity of ECU calibration programming sessions
    calLevel: CAL2
    securityProperty: authenticity
    threatScenarios:
      - TS-ENG-002

  - id: CSG-ENG-003
    name: Ensure firmware update integrity and rollback prevention
    calLevel: CAL3
    securityProperty: integrity
    threatScenarios:
      - TS-ENG-003

  - id: CSG-ENG-004
    name: Restrict diagnostic memory read access to authorised sessions
    calLevel: CAL2
    securityProperty: confidentiality
    threatScenarios:
      - TS-ENG-004

controlTable:
  - id: SC-ENG-001
    name: Implement AUTOSAR SecOC message authentication on powertrain CAN PDUs
    controlType: prevention
    implementsGoals:
      - CSG-ENG-001

  - id: SC-ENG-002
    name: Require cryptographic challenge-response for UDS programming sessions
    controlType: prevention
    implementsGoals:
      - CSG-ENG-002

  - id: SC-ENG-003
    name: Require cryptographic signature verification (ECDSA P-256) on firmware images before flash programming
    controlType: prevention
    implementsGoals:
      - CSG-ENG-003

  - id: SC-ENG-004
    name: Restrict UDS readMemoryByAddress to security access level 0x11/0x12 with session audit logging
    controlType: prevention
    implementsGoals:
      - CSG-ENG-004
---

## Scope

CAN bus powertrain network and OBD-II diagnostic port of the Engine ECU.
Assets in scope: safety-critical CAN PDUs (torque request, brake request),
ECU calibration data, UDS programming sessions.

## Methodology

Threat analysis per ISO/SAE 21434 §15. Attack feasibility rated using
ISO/SAE 21434 Annex B criteria (elapsed time, expertise, knowledge, window
of opportunity, equipment).
