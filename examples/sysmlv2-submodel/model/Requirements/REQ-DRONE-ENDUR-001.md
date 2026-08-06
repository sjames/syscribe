---
type: Requirement
id: REQ-DRONE-ENDUR-001
name: "Minimum flight endurance under nominal load"
status: approved
reqDomain: hardware
tags:
  - propulsion
  - endurance
---

The rotor assembly shall sustain continuous thrust output for a minimum of 20 minutes under nominal payload and wind conditions.

## Rationale

Endurance is a physical property of the rotor/motor/battery propulsion chain, not the flight-control software — the demo's SysML v2 `RotorAssembly` part def satisfies this directly (`satisfy 'REQ-DRONE-ENDUR-001';`, the quoted-id form of `REQ-TRS-SYSMLV2-003`).
