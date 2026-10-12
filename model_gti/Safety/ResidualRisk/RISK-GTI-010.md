---
type: ResidualRisk
id: RISK-GTI-010
name: "Gate drive stuck on and both relays welded"
status: proposed
faultSet: [FTE-GTI-001, FTE-GTI-002, FTE-GTI-003]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-038, REQ-GTI-HW-002]
reviewDate: 2026-10-12
expires: 2027-10-12
---

No actuator is left that can isolate the grid. Three independent hardware failures; the relays' welding is a latent fault that the weld detection before every connection (REQ-GTI-HW-002) reveals, so both relays must weld within one connection.
