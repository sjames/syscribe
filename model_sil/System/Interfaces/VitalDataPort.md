---
type: PortDef
name: Vital Data Port
---

Inter-channel vital data exchange port. This port carries the channel output state vector from one vital processor channel to the other for cross-comparison in the 2oo2D voting architecture.

The output state vector contains the complete set of commanded states for all controlled objects: signal aspects, points positions, relay energisation commands, and the channel's internal safety status word. Each vector is framed per EN 50159 Category 2 with a 32-bit CRC and a monotonically incrementing sequence number.

At the end of every scan cycle, each channel transmits its computed output state vector through this port and simultaneously receives the other channel's vector. The two vectors are compared bit-for-bit by the voting logic. Any discrepancy drives both channels to the safe state within the current scan cycle.

The port operates at the cross-comparison bus interface, which is electrically isolated from both channels' main buses to prevent a failure on one channel's bus from corrupting the other channel's state.
