---
type: ResidualRisk
id: RISK-GTI-001
name: "Supervisor and hardware trip both silent"
status: proposed
faultSet: [FTE-GTI-005, FTE-GTI-006]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-071, REQ-GTI-SYS-039]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The software path and the independent hardware trip fail together, so nothing stops the bridge or opens the relays. A dual-point fault: the supervisor failure is detected by the software self-test (REQ-GTI-SYS-071), which leaves the hardware trip's own latent failure as the second fault. Proposed as residual risk on the strength of the independence of the two paths.
