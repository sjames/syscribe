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
* **Software did not act (FTG-GTI-004):** the supervisor is silent, or both software detections missed the loss (FTG-GTI-009): the voltage/frequency monitor (silent, or blinded by its sensing chain, FTG-GTI-010) **and** the perturbation-response check (FTE-GTI-010).
* The two voltage-sensing chains are separate events (FTE-GTI-007, FTE-GTI-008). A **common cause** that blinds both (a wiring error at the grid connection point) is the basic event FTE-GTI-009, which feeds both *monitor missed the loss* and *hardware trip failed*. The perturbation-response check (REQ-GTI-SYS-045, REQ-GTI-SW-008) does not rely on the absolute voltage: it injects a small current and measures how the grid answers, through the software sensing chain, so a chain that reads a fixed value shows no answer and the check reports the grid lost. FTE-GTI-009 is therefore a cut set of order 2 together with the failure of the check (FTE-GTI-010), and the campaign agrees: the single event survives, the pair is defeated. The grid's answer is an in-process plant (`Behavior::Plants::GridImpedance`, GH #340). Loss of a command (dropped monitor or relay message, FM-GTI-002 and FM-GTI-004) is a form of *software did not act* and is covered by FTE-GTI-004 and FTE-GTI-005.

Mission time 131 400 h is 15 years. Failure rates are illustrative proposals, **unverified**, for architecture comparison only.
