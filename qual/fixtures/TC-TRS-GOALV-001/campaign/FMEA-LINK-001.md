---
type: FMEASheet
id: FMEA-LINK-001
name: "FMEA of the link"
status: draft
entries:
  - {id: FM-TX-003, ref: TxP, failureMode: "level stuck", ftaRef: FTE-SYS-001, injection: [{kind: stuck, variable: level, value: "3"}]}
  - {id: FM-TX-004, ref: TxP, failureMode: "level biased", injection: [{kind: offset, variable: level, by: "2"}]}
---
