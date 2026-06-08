---
type: PartDef
name: SafetyCore
domain: software
asilLevel: D
breakdownAdr: ADR-FFI-PART-001
allocatedTo:
  - HostECU
---

ASIL D safety-critical software component allocated to the shared host ECU. The
`breakdownAdr` points to an accepted partitioning ADR, excusing the sharing.
