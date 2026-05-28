---
type: Part
name: Vital Processor Channel A
typedBy: System::Hardware::VitalProcessor
domain: hardware
---

Channel A of the 2oo2D vital processor at Station 1 Interlocking. This channel executes the complete interlocking vital logic independently from channel B and transmits its output state vector to channel B via the VitalSafetyLink cross-comparison bus at the end of every scan cycle.

Channel A is designated the primary channel for signaller workstation communication; it forwards validated route requests to both channels. Channel A also hosts the clock master for the inter-channel synchronisation protocol.
