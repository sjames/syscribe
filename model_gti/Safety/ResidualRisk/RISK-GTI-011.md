---
type: ResidualRisk
id: RISK-GTI-011
name: "Both sensing chains blinded: a dead grid is energised until the perturbation check trips"
status: proposed
faultSet: [FTE-GTI-009]
goal: SG-GTI-001
mitigatedBy: [REQ-GTI-SYS-045, REQ-GTI-SW-008]
reviewDate: 2026-10-12
expires: 2027-10-12
---

With both voltage-sensing chains reading nominal (the common cause FTE-GTI-009), the supervisor sees a healthy grid at power-up and connects to a dead one when the reconnect delay ends. The perturbation-response check (REQ-GTI-SYS-045, REQ-GTI-SW-008) needs the bridge to switch, so it runs only once the inverter is connected; it reports the grid lost 1.1 s later and the chain trips. The invariant "a dead grid at power-up keeps the safe state" states that the safe state is never left, so the single fault defeats it, although the energisation ends within the 2 s island-clearing time of SG-GTI-001.

This defeat appeared when the invariants were rewritten with `always` (GH #342): the earlier invariant was checked only at the end of the scenario, after the trip. The alternative to accepting it is to run the check before connecting (a perturbation at standby, without export), which the compliance engineer should weigh.
