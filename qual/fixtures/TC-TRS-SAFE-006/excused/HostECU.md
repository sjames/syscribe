---
type: PartDef
name: HostECU
domain: hardware
ffiRationale: "MPU spatial partitioning isolates the ASIL D and QM partitions; timing protection bounds interference (ISO 26262-6 §7.4.9)."
---

Shared host ECU hardware. The `ffiRationale` documents the freedom-from-interference
argument, so the mixed-criticality SafetyCore/Infotainment sharing is excused.
