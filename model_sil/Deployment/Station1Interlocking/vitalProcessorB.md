---
type: Part
name: Vital Processor Channel B
typedBy: System::Hardware::VitalProcessor
domain: hardware
---

Channel B of the 2oo2D vital processor at Station 1 Interlocking. This channel executes the complete interlocking vital logic independently from channel A and transmits its output state vector to channel A via the VitalSafetyLink cross-comparison bus at the end of every scan cycle.

Channel B runs diverse software compiled with a different toolchain or on a diverse hardware revision to achieve the software diversity required for the 2oo2D architecture. Channel B monitors channel A's watchdog heartbeat and initiates the safe state sequence if channel A's heartbeat is absent for more than one scan cycle.
