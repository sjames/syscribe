---
id: HE-SIL-001
type: HazardousEvent
status: approved
name: Signal cleared into occupied track section — risk of rear-end collision
consequence: Cd
freqExposure: Fb
avoidance: Pb
demandRate: W3
operationalSituation: Normal signalled operation on a busy multi-train corridor, any speed
---

A signal is cleared to proceed while the section ahead is occupied by a stationary or slower-moving train. The following driver has no independent means of detecting the occupied section before the collision point. At line speeds above 100 km/h, stopping distance exceeds the overlap.

Outcome: collision with potential for multiple fatalities and track infrastructure damage. The hazard arises from the interlocking failing to enforce block separation — the fundamental safety obligation of any signalling system.

Risk parameters per IEC 61508-5 Annex D risk graph:

- **Consequence (Cd)**: Catastrophic — multiple fatalities and major infrastructure damage are expected at typical line speeds.
- **Frequency of exposure (Fb)**: Frequent — trains operate continuously throughout the service day; the hazardous situation (occupied section ahead of a signal) occurs as a routine operational state.
- **Avoidance (Pb)**: Not possible — the driver cannot independently detect an occupied section at speed without the signalling system; no independent protection within the overlap distance.
- **Demand rate (W3)**: High — the interlocking is demanded continuously on a busy corridor.

**SIL target: 4** (Cd × Fb × Pb × W3 per IEC 61508-5 Annex D).
