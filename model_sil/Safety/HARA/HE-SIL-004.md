---
id: HE-SIL-004
type: HazardousEvent
status: approved
title: Train signal cleared while level crossing barriers not down — road/rail collision
consequence: Cc
freqExposure: Fb
avoidance: Pa
demandRate: W2
operationalSituation: Level crossing approach with road traffic present
---

A train movement signal is cleared while the level crossing barriers are not confirmed in the lowered position. A road vehicle traversing the crossing at the same time as the train is at extreme risk of collision. This hazard differs from the other CBI hazardous events in that road users have some residual ability to see or hear an approaching train, providing a partial avoidance mechanism not available in track-section conflicts.

The hazard arises from: barrier position sensor failure reporting confirmed-down when barriers are still rising or have not completed their stroke; LevelCrossingModule software error that does not gate signal clearance on barrier confirmation; or race condition between barrier lowering command and signal clearance authorization.

Risk parameters per IEC 61508-5 Annex D risk graph:

- **Consequence (Cc)**: Critical — a road/rail collision at a level crossing is likely to be fatal to road vehicle occupants; the train itself is generally not derailed but may sustain significant damage.
- **Frequency of exposure (Fb)**: Frequent — level crossings are operated continuously throughout the service day at every train movement past the crossing.
- **Avoidance (Pa)**: Possible under specific conditions — a road user may observe or hear the approaching train and stop before the crossing; this partial avoidance possibility distinguishes this event from HE-SIL-001/002 (hence Pa rather than Pb and a SIL 3 rather than SIL 4 target).
- **Demand rate (W2)**: Moderate — level crossing operations are frequent but less continuous than mainline block operations.

**SIL target: 3** (Cc × Fb × Pa × W2 per IEC 61508-5 Annex D risk graph).
