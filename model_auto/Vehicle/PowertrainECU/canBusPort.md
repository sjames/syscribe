---
type: Port
name: CAN Bus Port
typedBy: System::Interfaces::CANBusPort
---

Powertrain CAN bus connector on the ECU harness — a 2-pin differential pair (CANH, CANL)
terminated at the ECU housing connector (2-pin Superseal 1.5 or Bosch econoseal equivalent,
rated to IP67).

## Electrical characteristics

- Bus topology: twisted-pair, 120 Ω characteristic impedance
- Bit rate: 500 kbit/s; bit time 2 µs (16 time quanta at 8 MHz CAN clock)
- Common-mode voltage range: −2 V to +7 V (ISO 11898-2 §6.6)
- Differential voltage in dominant state: 1.5 V minimum (CANH − CANL)
- Maximum stub length at this node: 300 mm (meets ISO 11898-2 signal integrity requirements)

## Frame filtering

The AUTOSAR CanIf layer configures hardware acceptance filters on the MCU CAN peripheral.
Only PDUs with arbitration IDs in the range 0x000–0x0FF (powertrain control frames) and
0x700–0x7FF (diagnostic frames) are passed to the application. All other frames are discarded
in hardware, reducing interrupt load and preventing unintended cross-bus message processing.
