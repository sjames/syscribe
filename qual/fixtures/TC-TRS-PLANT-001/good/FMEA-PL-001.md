---
type: FMEASheet
id: FMEA-PL-001
name: "FMEA of the lag"
status: draft
entries:
  - {id: FM-PL-001, ref: LagP, failureMode: "gain doubled", injection: [{kind: parameter, parameter: k, value: "2"}]}
  - {id: FM-PL-002, ref: LagP, failureMode: "output stuck", injection: [{kind: stuck, variable: y, value: "0.5"}]}
---
