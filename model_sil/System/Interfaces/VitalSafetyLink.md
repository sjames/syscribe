---
type: ConnectionDef
name: Vital Safety Link
ends:
  - name: channelA
    typedBy: System::Interfaces::VitalDataPort
  - name: channelB
    typedBy: System::Interfaces::VitalDataPort
---

Binary safety connection between channel A and channel B of the 2oo2D vital processor. Each channel transmits its full output state vector to the other channel over this link at the end of every scan cycle. Both channels must agree on the computed output before any output is asserted.

The link is implemented as a dedicated cross-comparison bus, physically separate from both channels' internal buses and from the field bus. The bus is unidirectional at each end (channel A writes to channel B's input register; channel B writes to channel A's input register) using dual-port RAM or equivalent isolation.

The VitalSafetyLink operates within the SafetyCommLayer's EN 50159 Category 2 protocol envelope, providing CRC checking, sequence number validation, and timeout detection for the inter-channel communication path. Any failure detected by the SafetyCommLayer on this link drives both channels to the safe state within one scan cycle.
