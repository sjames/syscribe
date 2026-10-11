---
type: PartDef
name: DC Input
domain: hardware
satisfies:
  - REQ-GTI-SYS-034
  - REQ-GTI-SYS-035
  - REQ-GTI-HW-007
features:
  - {name: pvIn, type: Port, direction: in}
  - {name: dcOut, type: Port, direction: out}
  - {name: strSense, type: Port, direction: out}
---

Two string inputs per tracker with 1000 V DC fuses, reverse-current diodes, a load-break DC switch, type-II DC surge protection and 1100 V-rated capacitors and semiconductors.
