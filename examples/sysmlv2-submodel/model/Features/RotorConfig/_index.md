---
type: FeatureDef
id: FEAT-ROTOR-CONFIG
name: RotorConfig
groupKind: alternative
mandatory: true
---

Rotor configuration of the airframe. Every product selects **exactly one** rotor
variant (XOR group) — targeted from the SysML v2 side by the `RotorConfigChoice`
variation part def's `variant` members, each carrying a `@SyscribeFeature`
metadata annotation (`REQ-TRS-SYSMLV2-005`).
