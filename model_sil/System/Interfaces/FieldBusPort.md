---
type: PortDef
name: Field Bus Port
---

EN 50159 Category 2 field bus port connecting the vital processor to the Object Controllers in the field. The field bus operates at 1 Mbit/s over industrial Ethernet (IEC 61784-3 profile) with deterministic polling at a 20 ms cycle time matching the vital processor scan period.

Each Object Controller is addressed individually by the vital processor in a master-slave polling scheme. The vital processor initiates every exchange; Object Controllers respond only when polled. This architecture ensures that a faulty Object Controller cannot inject spurious commands or status messages into the field bus.

All messages on the field bus are protected by EN 50159 Category 2 safety codes: 32-bit CRC, sequence number, timestamp, and connection identifier. The SafetyCommLayer is responsible for encoding and decoding these codes at both ends.

The field bus is electrically isolated from the vital processor board and from the trackside field circuits, providing two levels of galvanic isolation. The bus uses screened twisted-pair cabling with impedance-matched termination to meet the EMC requirements of EN 50121.
