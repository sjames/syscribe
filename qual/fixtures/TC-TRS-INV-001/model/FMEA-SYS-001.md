---
type: FMEASheet
id: FMEA-SYS-001
name: "FMEA of the protection"
status: draft
entries:
  - {id: FM-SYS-001, ref: MP, failureMode: "a welded", injection: [{kind: stuck, variable: a, value: "1"}]}
  - {id: FM-SYS-002, ref: MP, failureMode: "b welded", injection: [{kind: stuck, variable: b, value: "1"}]}
  - {id: FM-SYS-003, ref: MP, failureMode: "alarm dead", injection: [{kind: stuck, variable: c, value: "0"}]}
---
