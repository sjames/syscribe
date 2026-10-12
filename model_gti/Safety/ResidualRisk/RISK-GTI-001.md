---
type: ResidualRisk
id: RISK-GTI-001
name: "Supervisor and hardware trip both silent"
status: proposed
faultSet: [FTE-GTI-005, FTE-GTI-006]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-039, REQ-GTI-SYS-071]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The software path and the independent hardware trip fail together, so nothing stops the bridge or opens the relays. A dual-point fault of two independent channels (REQ-GTI-SYS-039 keeps the hardware trip independent of the software). The software self-test (REQ-GTI-SYS-071) is meant to reveal a hung supervisor, but the executable model has no self-test, so the campaign gives it no credit: the residual risk rests on the independence of the two paths and on the rates of FT-GTI-001.
