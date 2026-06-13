---
type: FMEASheet
id: FMEA-XREF-001
name: "Cross-reference fixture for fmeaRef/ftaRef testing"
status: draft
entries:
  - id: FM-KERN-001
    failureMode: "Null pointer dereference"
    effect: "System crash"
    fmeaSeverity: 7
    occurrence: 4
    detection: 3
    ftaRef: FTE-KERN-001
  - id: FM-NONEXIST-REF
    failureMode: "Missing FTE ref"
    effect: "No FTE"
    fmeaSeverity: 3
    occurrence: 2
    detection: 2
    ftaRef: FTE-NONEXIST-001
---
