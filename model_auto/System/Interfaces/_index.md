---
type: Package
name: Interfaces
---

Port and interface type definitions for the Engine ECU external connections. These type
definitions are used to type the `Port` usages on `PartDef` elements and to constrain
which `Connection` elements may link them.

## Contents

| Element | Type | Description |
|---|---|---|
| `CANBusPort` | PortDef | Typed endpoint for ISO 11898-2 high-speed CAN connections |
| `CANBusConnection` | ConnectionDef | Typed CAN link; `transmitter` and `receiver` ends |
| `CANFrame` | ItemDef | CAN frame payload item flowing through `CANBusConnection` |
| `OBDIIPort` | PortDef | Typed endpoint for ISO 15031 OBD-II diagnostic access |
| `PowertrainCANInterface` | InterfaceDef | Powertrain CAN bus interface with 500 kbit/s and SecOC constraints |

## CAN bus naming convention

The ECU has two logical CAN roles: the `canOut` port on `ThrottleControl` SWC (produces
`TorqueCommandFrame` items) and the `canIn` port on `CANTransceiver` (receives serialised
frames from the MCU TXD line). The `CANBusConnection` in the `Vehicle::PowertrainECU`
connects these two endpoints.

## Security on OBD-II

The `OBDIIPort` is the physical entry point for UDS diagnostic sessions. All programming
access through this port is subject to SC-ENG-002 (seed-and-key), SC-ENG-003 (firmware
signature), and SC-ENG-004 (memory read access control). Physical access to the OBD-II
connector is itself a TARA threat vector (see `Security/TARA-ENG-001`).
