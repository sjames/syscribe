---
type: ResidualRisk
id: RISK-GTI-002
name: "Supervisor silent and hardware-path sensing reads nominal"
status: proposed
faultSet: [FTE-GTI-005, FTE-GTI-008]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-HW-008, REQ-GTI-SYS-039]
reviewDate: 2026-10-12
expires: 2027-10-12
---

The supervisor cannot act and the hardware trip is blinded by its own sensing chain. A dual-point fault of two independent channels; the hardware trip has its own sensing (REQ-GTI-HW-008) so that one sensing failure blinds only one channel. The software self-test (REQ-GTI-SYS-071) is not modelled and is given no credit.
