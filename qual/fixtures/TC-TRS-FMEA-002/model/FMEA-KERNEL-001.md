---
type: FMEASheet
id: FMEA-KERNEL-001
name: "Kernel FMEA fixture"
status: draft
entries:
  - id: FM-KERN-001
    failureMode: "Null pointer dereference"
    effect: "System crash"
    cause: "Unvalidated pointer"
    fmeaSeverity: 9
    occurrence: 9
    detection: 9
  - id: FM-KERN-002
    failureMode: "Buffer overflow"
    effect: "Memory corruption"
    cause: "Missing bounds check"
    severity: 8
    occurrence: 4
    detection: 3
  - id: FM-KERN-003
    failureMode: "Race condition"
    effect: "Data corruption"
    cause: "Missing mutex"
    failureEffect: "System deadlock"
    fmeaSeverity: 6
    occurrence: 3
    detection: 5
---

Kernel FMEA for validation testing.
