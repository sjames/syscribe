---
type: FMEASheet
id: FMEA-ROWS-001
name: "Row-integrity fixture: missing id and inconsistent rpn"
status: draft
entries:
  - id: FM-ROWS-001
    failureMode: "Consistent explicit rpn"
    effect: "None"
    fmeaSeverity: 2
    occurrence: 3
    detection: 4
    rpn: 24
  - failureMode: "Orphan row with no id"
    effect: "Vanishes from the analysis"
    fmeaSeverity: 9
    occurrence: 9
    detection: 9
  - id: FM-ROWS-003
    failureMode: "Stale explicit rpn"
    effect: "Wrong priority"
    fmeaSeverity: 5
    occurrence: 4
    detection: 3
    rpn: 100
    recommendedAction: "Recompute"
  - id: FM-ROWS-004
    failureMode: "Partial factors with explicit rpn"
    effect: "Only rpn known"
    fmeaSeverity: 5
    occurrence: 4
    rpn: 80
---

Fixture for TC-TRS-FMEA-004.
