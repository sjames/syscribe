---
type: ResidualRisk
id: RISK-GTI-006
name: "Software detections silent and hardware-path sensing reads nominal"
status: proposed
faultSet: [FTE-GTI-004, FTE-GTI-008, FTE-GTI-010]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-HW-008, REQ-GTI-SW-008]
reviewDate: 2026-10-12
expires: 2027-10-12
---

Three independent failures that remove both software detections and blind the hardware trip through its own sensing chain (REQ-GTI-HW-008). Beyond the single-fault tolerance of SG-GTI-001.
