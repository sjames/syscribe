---
type: Part
name: CAN Transceiver
typedBy: System::Hardware::CANTransceiver
domain: hardware
---

ISO 11898-2 high-speed CAN transceiver IC (TJA1044 or pin-compatible equivalent) on the
Engine ECU PCB. Converts the MCU's 3.3 V logic TXD/RXD signals to the 5 V differential
CANH/CANL bus signals specified by ISO 11898-2.

## Bus parameters

Nominal bit rate: 500 kbit/s on the powertrain CAN backbone. Bus termination: 120 Ω at
each network end (not on the ECU node). The ECU node presents a capacitive stub load of
< 100 pF as required by ISO 11898-2 for 33 nodes maximum network topology.

## Fault protection

The TJA1044 provides:
- **Bus fault protection** — withstands ±36 V common-mode (ISO 7637-2 pulse 1/2 transients).
- **TXD dominant timeout** — if MCU TXD is held dominant for more than 4 ms the transceiver
  automatically releases the bus to prevent bus blocking, which would interrupt all ECU-to-CAN
  communication and trigger W803-class diagnostic events.
- **Thermal shutdown** — protects the IC at die temperatures > 150 °C.

## AUTOSAR integration

The transceiver is managed by the AUTOSAR CAN Driver (CanDrv) and CAN Interface (CanIf)
modules in the BSW layer. The `CANSecurityModule` SWC inserts SecOC MACs into the CanIf
PDU before transmission; the hardware transceiver is unaware of the security layer.
