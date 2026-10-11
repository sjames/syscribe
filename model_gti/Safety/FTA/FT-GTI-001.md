---
id: FT-GTI-001
type: FaultTree
name: Fault tree for SG-GTI-001 — inverter energises a de-energised grid
topEvent: SG-GTI-001
status: draft
missionTime: "131400 h"
---

Top event: after loss of the grid the bridge is still switching **and** both series relays are still closed beyond the 2 s island-clearing time.

* **Gates on (FTG-GTI-002):** the gate drive is stuck on, or software did not shut it down **and** the hardware trip also failed.
* **Relays closed (FTG-GTI-003):** relay A cannot isolate (welded, or software did not command it) **and** relay B cannot isolate (welded, or software did not command it **and** the hardware trip failed).
* **Software did not act (FTG-GTI-004):** the monitor is silent or the supervisor is silent.
* The two voltage-sensing chains are separate events (FTE-GTI-007, FTE-GTI-008). A **common cause** that blinds both (a wiring error at the grid connection point) is **not modelled yet**: it needs a `ccfGroup` event and is an open item. Loss of a command (dropped monitor or relay message, FM-GTI-002 and FM-GTI-004) is a form of *software did not act* and is covered by FTE-GTI-004 and FTE-GTI-005.

Mission time 131 400 h is 15 years. Failure rates are illustrative proposals, **unverified**, for architecture comparison only.
