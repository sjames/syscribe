---
id: SG-SIL-003
type: SafetyGoal
status: approved
title: Prevent train signal clearance while level crossing barriers are not confirmed down
silLevel: 3
safeState: Train signal at most-restrictive aspect; level crossing warning active
ftti: 500ms
hazardousEvents:
  - HE-SIL-004
---

The LevelCrossingModule (System::Hardware::LevelCrossingModule) **shall** confirm barriers in the lowered position before the SignalController is permitted to clear any signal for a route that includes or approaches the level crossing. The confirmation must be a positively proved input — detection current present — not merely the absence of a "barrier up" indication. Negative logic (absence of fault) is not sufficient for safety confirmation.

**Confirmation architecture.** Each barrier must provide two independent detection circuits (primary and secondary), both of which must simultaneously confirm the lowered position. Detection circuits use separate cable pairs, separate relay circuits, and separate power supplies. A failure of any detection circuit shall cause the barrier to be treated as not confirmed and shall prevent signal clearance.

**Independence from barrier drive.** The barrier lowering command and the barrier position confirmation are functionally independent: the LevelCrossingModule issues the lowering command and separately reads back the confirmed position. A failure in the drive circuit that prevents lowering will also prevent confirmation, causing the signal to remain at red — this is a safe-side failure. A failure in the detection circuit that falsely confirms a lowered position while barriers are up is the dangerous failure mode addressed by the two-circuit architecture.

**SIL 3 target.** This goal carries a lower integrity level than SG-SIL-001 and SG-SIL-002 because the corresponding hazardous event (HE-SIL-004) has avoidance parameter Pa rather than Pb — road users retain a partial ability to perceive and avoid an approaching train. The SIL 3 dangerous failure rate target is < 10⁻⁷ /h (IEC 61508 Table 2).

**Fault tolerance time interval (FTTI): 500 ms.** Longer than the mainline FTTI because level crossing approach speeds are lower (typically ≤ 160 km/h at crossings) and the approach section provides more warning distance. The 500 ms bound permits one or two additional scan cycles for confirmation debounce before the signal is cleared.
