---
type: FMEASheet
id: FMEA-LINK-001
name: "FMEA of the link"
status: draft
entries:
  - {id: FM-TX-001, ref: TxP, failureMode: "command lost", injection: [{kind: drop, port: out}]}
  - {id: FM-TX-003, ref: TxP, failureMode: "level stuck", injection: [{kind: stuck, variable: level, value: "3"}]}
  - {id: FM-TX-004, ref: TxP, failureMode: "level biased", injection: [{kind: offset, variable: level, by: "2"}]}
---
