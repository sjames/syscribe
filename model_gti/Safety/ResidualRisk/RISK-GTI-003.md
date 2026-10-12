---
type: ResidualRisk
id: RISK-GTI-003
name: "Supervisor silent and both sensing chains blinded"
status: proposed
faultSet: [FTE-GTI-005, FTE-GTI-009]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-045, REQ-GTI-SW-008]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The common cause that blinds both sensing chains removes the hardware trip; with the supervisor also silent, the perturbation-response check (REQ-GTI-SYS-045, REQ-GTI-SW-008) has no one to act on its finding. A dual-point fault.
