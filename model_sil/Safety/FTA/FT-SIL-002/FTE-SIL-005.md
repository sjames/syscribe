---
id: FTE-SIL-005
type: FaultTreeEvent
name: FTE-SIL-005
eventKind: basic
title: Points detection false-confirm — points not at commanded position but detection reports confirmed
ref: System::Hardware::PointsDriveModule
failureRate: 8.0e-10
probability: 8.0e-7
---

The points detection contacts fail in a way that indicates position confirmation when the points are not actually in the commanded position. This is a dangerous hardware failure in the trackside detection circuit — the CBI receives a "points confirmed" signal from the PointsDriveModule while the physical switch blade is in an intermediate, open, or incorrect position.

**Failure mechanism.** Points detection uses mechanical contacts (or in modern systems, inductive proximity sensors or encoder outputs) that change state when the switch blade reaches the end-of-stroke position. The dangerous failure mode is a contact or sensor that becomes stuck in the "confirmed" state while the physical blade position does not match:

1. **Contact welded closed**: High contact current during a points operation can weld the detection relay contacts in the closed position. Subsequent operations that fail to fully drive the points will still produce a "confirmed" output.
2. **Wire bridge across the detection circuit**: An insulation failure between the detection circuit conductors can bridge the contact, producing a permanent "confirmed" indication regardless of blade position.
3. **Sensor offset drift**: In proximity sensor systems, sensor mounting offset or mechanical wear can cause the sensor to trigger before the blade has reached its correct end position.

**Defence by circuit separation.** The drive and detection circuits use separate cable pairs and separate relay circuits, making a single failure less likely to affect both. The detection contact series architecture (two independent contacts per detection position) is the primary mitigation — both contacts must close for confirmation, so a single welded contact does not produce a false confirmation if the second contact is not welded.

**Failure rate basis.** 8.0 × 10⁻¹⁰ /h accounts for the two-contact series architecture; the single-contact failure rate is approximately 4.0 × 10⁻⁹ /h, and the series combination reduces the dangerous failure rate to the quoted value, assuming independence of the two contacts.
