---
type: ResidualRisk
id: RISK-GTI-004
name: "Both sensing chains blinded and the perturbation check silent"
status: proposed
faultSet: [FTE-GTI-009, FTE-GTI-010]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-045, REQ-GTI-SW-008]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The perturbation-response check (REQ-GTI-SYS-045, REQ-GTI-SW-008) is the defence against the common cause FTE-GTI-009; this is that defence failing as well. A dual-point fault, and the order-2 cut set FT-GTI-001 names for the common cause.
