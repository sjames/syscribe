---
type: Part
name: Station 1 Interlocking
typedBy: System::InterlockingSystem
domain: hardware
connections:
  - typedBy: System::Interfaces::VitalSafetyLink
    from: vitalProcessorA.vitalDataPort
    to: vitalProcessorB.vitalDataPort
---

The installed interlocking at a typical two-platform station with a facing junction. This deployment controls a track layout consisting of:

- Two diverse vital processor boards (channel A and channel B) in 2oo2D configuration
- A facing junction with two points machines (normal and trailing)
- Three track circuit sections (approach, platform 1, platform 2)
- Two home signals (down direction and up direction)
- One level crossing (LC-001) on the approach to the station

The interlocking manages approach locking from both directions: an approaching train on any approach track circuit causes the relevant route to become approach-locked, preventing cancellation until the train has passed the signal or a time-lock procedure has been completed.

The two Object Controllers provide field connectivity: OC1 serves the home signals, and OC2 serves the points machines and the level crossing module. Both OCs are connected to the vital processor via the EN 50159 Category 2 field bus.

The vital processor channels are connected to each other via the VitalSafetyLink cross-comparison bus as specified by the VitalSafetyLink ConnectionDef.
