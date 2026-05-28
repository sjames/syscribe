---
type: PartDef
name: CAN Transceiver
domain: hardware
features:
  - name: standard
    type: ScalarValues::String
  - name: busSpeedKbps
    type: ScalarValues::Integer
    unit: kbit/s
---

ISO 11898-2 high-speed CAN transceiver providing physical-layer interface
between the ECU microcontroller CAN peripheral and the vehicle CAN bus.

Includes bus fault protection (short-to-battery, short-to-ground), and
TXD-dominant timeout to prevent bus blocking on ECU software failure.
