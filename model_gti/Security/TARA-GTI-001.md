---
type: TARASheet
id: TARA-GTI-001
name: "TARA — Inverter remote interfaces and firmware update"
status: draft
date: "2026-10-11"
damageTable:
  - id: DS-GTI-001
    name: "Disabled or altered protection settings lead to energising a de-energised grid"
    damageSeverity: severe
    impactCategories: [safety, operational]
    hazardRef: SG-GTI-001
    assets: [ASSET-GTI-001]
  - id: DS-GTI-002
    name: "Coordinated curtailment or region change of many inverters destabilises a local grid"
    damageSeverity: major
    impactCategories: [operational, financial]
    assets: [ASSET-GTI-002]
  - id: DS-GTI-003
    name: "Malicious firmware turns the inverter into a hazard or a bot"
    damageSeverity: severe
    impactCategories: [safety, operational, financial]
    hazardRef: SG-GTI-001
    assets: [ASSET-GTI-003]
threatTable:
  - id: TS-GTI-001
    name: "Attacker writes unauthenticated Modbus registers to change the grid profile or trip limits"
    attackFeasibility: medium
    attackVector: adjacent
    damageScenarios: [DS-GTI-001]
  - id: TS-GTI-002
    name: "Attacker replays or forges IEEE 2030.5 / cloud curtailment commands at scale"
    attackFeasibility: medium
    attackVector: network
    damageScenarios: [DS-GTI-002]
  - id: TS-GTI-003
    name: "Attacker installs modified firmware through the update channel or by debug access"
    attackFeasibility: low
    attackVector: network
    damageScenarios: [DS-GTI-003]
goalTable:
  - id: CSG-GTI-001
    name: "Only authenticated, authorised parties can change protection settings or the region profile"
    calLevel: CAL4
    securityProperty: authenticity
    threatScenarios: [TS-GTI-001, TS-GTI-002]
  - id: CSG-GTI-002
    name: "Only manufacturer-signed, non-rolled-back firmware runs on the STM32"
    calLevel: CAL3
    securityProperty: integrity
    threatScenarios: [TS-GTI-003]
controlTable:
  - id: SC-GTI-001
    name: "Authenticated and integrity-protected commands; read-only default for Modbus; service credential for profile changes"
    controlType: prevention
    implementsGoals: [CSG-GTI-001]
  - id: SC-GTI-002
    name: "Secure boot with signature check, anti-rollback counter, and STM32 readout protection / debug lock"
    controlType: prevention
    implementsGoals: [CSG-GTI-002]
---

Scope: the remote interfaces (Modbus, IEEE 2030.5 client, local service port) and the firmware update path. Out of scope for this version: supply-chain and physical tampering beyond debug lock. Linked requirements: REQ-GTI-SYS-053, REQ-GTI-SYS-054, REQ-GTI-SYS-060.
