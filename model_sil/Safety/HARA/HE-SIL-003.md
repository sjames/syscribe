---
id: HE-SIL-003
type: HazardousEvent
status: approved
title: Signal cleared with points not in proved position — risk of derailment
consequence: Cc
freqExposure: Fb
avoidance: Pb
demandRate: W3
operationalSituation: Route-setting through a junction with points not at detected position
---

A signal is cleared for a route through a junction while the points machines are moving, have failed mid-stroke, or detection contacts have not yet made. A train traversing mis-set points will derail, with risk of injury or fatality to passengers and potential secondary collision with infrastructure or other trains.

The hazard can arise from: points detection contact failure in the confirmed position while points are actually open; timing race between drive command and detection confirmation; detection relay contact welded in the confirmed position; or software error in the SignalController that clears a signal before all detection conditions are simultaneously satisfied.

Risk parameters per IEC 61508-5 Annex D risk graph:

- **Consequence (Cc)**: Critical — derailment at speed is likely to injure or kill passengers and crew; multiple casualties expected at line speed; lower than Cd because the derailment may not involve a full-speed collision with another train.
- **Frequency of exposure (Fb)**: Frequent — points operations occur continuously throughout the service day at every junction.
- **Avoidance (Pb)**: Not possible — the driver cannot detect mis-set points before the derailment point; no independent protection available.
- **Demand rate (W3)**: High — points position confirmation is demanded on every route-setting operation.

**SIL target: 4** (Cc × Fb × Pb × W3 per IEC 61508-5 Annex D).
