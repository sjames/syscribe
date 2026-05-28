---
type: Package
name: Hardware
---

Hardware sub-components integrated on the Engine ECU printed circuit board. These PartDefs
represent discrete ICs or functional blocks that are distinct from the main microcontroller
and are modelled separately because they have independent failure modes and are subjects of
dedicated FMEA entries.

## Components

| PartDef | ASIL | Function | Failure impact |
|---|---|---|---|
| `CANTransceiver` | — | Converts MCU logic-level TXD/RXD to ISO 11898-2 differential signals | Loss of CAN communication; bus fault isolation |
| `WatchdogTimer` | D (external) | Independent hardware timer that resets the MCU if software fails to service it | Software lock-up recovery; ASIL D hardware channel |

## WatchdogTimer independence

The `WatchdogTimer` is an **external** IC (not a peripheral inside the MCU) to satisfy the
ISO 26262-6 §9 independence requirement for the ASIL D hardware channel of the ASIL D
decomposition. The reset output connects to both the MCU nRST pin and the throttle actuator
enable line, ensuring the throttle is disabled simultaneously with the MCU reset even if the
MCU is non-responsive.
