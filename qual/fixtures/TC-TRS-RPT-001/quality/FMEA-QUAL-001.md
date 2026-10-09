---
type: FMEASheet
id: FMEA-QUAL-001
name: "FMEA row quality"
status: approved
entries:
  - id: FM-QUAL-001
    failureMode: "Severe but unlikely"
    effect: "Loss of control"
    fmeaSeverity: 10
    occurrence: 2
    detection: 2
  - id: FM-QUAL-002
    failureMode: "Row with no scores"
    effect: "Unknown"
  - id: FM-QUAL-003
    failureMode: "Dangling subject"
    effect: "Minor"
    ref: Nowhere::Missing
    fmeaSeverity: 2
    occurrence: 2
    detection: 2
  - id: FM-QUAL-001
    failureMode: "Duplicate id"
    effect: "Duplicate"
    fmeaSeverity: 3
    occurrence: 3
    detection: 3
---
