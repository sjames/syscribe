---
type: ResidualRisk
id: RISK-GTI-008
name: "Both sensing chains read nominal independently and the perturbation check silent"
status: proposed
faultSet: [FTE-GTI-007, FTE-GTI-008, FTE-GTI-010]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-HW-008, REQ-GTI-SW-008]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The two sensing chains, kept independent by REQ-GTI-HW-008, fail independently (not the common cause FTE-GTI-009), and the perturbation check (REQ-GTI-SW-008) is silent as well. Three independent failures.
