---
type: ResidualRisk
id: RISK-GTI-009
name: "Gate drive stuck on, relay B welded and supervisor silent"
status: proposed
faultSet: [FTE-GTI-001, FTE-GTI-003, FTE-GTI-005]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-071, REQ-GTI-HW-002]
reviewDate: 2026-10-12
expires: 2027-10-12
---

With the gate drive stuck on and relay B welded, only relay A can isolate, and it waits for a software command that the silent supervisor never sends. Three independent failures.
