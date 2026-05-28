---
type: Package
name: System
---

This package contains the structural decomposition of the Engine ECU at the type (PartDef)
level. It defines the architecture vocabulary: hardware components, software functional
components, sensors, actuators, and interface types. Concrete instances are in `Vehicle/`.

## Sub-packages

| Package | Domain | Contents |
|---|---|---|
| `Sensors/` | hardware | Abstract `Sensor` base and three concrete sensor PartDefs |
| `Actuators/` | hardware | `ThrottleActuator` and `FuelInjector` PartDefs |
| `Hardware/` | hardware | `CANTransceiver` and `WatchdogTimer` board sub-components |
| `Interfaces/` | — | `CANBusPort`, `OBDIIPort`, `CANBusConnection`, `CANFrame`, `PowertrainCANInterface` |
| `Software/` | software | Eight software PartDefs (SWCs) and one ActionDef (BootSequence) |

## Domain independence

Per traceability rule R-006, `domain: software` and `domain: hardware` elements share no
`supertype:` or `typedBy:` links. Cross-domain interaction is modelled exclusively through:

1. `Allocation` elements in `Allocations/` (software → hardware deployment target)
2. `Port`/`Connection` elements at the ECU boundary (sensor signals, actuator commands)
3. `allocatedFrom:` links on architecture elements pointing to `SecurityControl` instances

## Top-level elements

| Element | Type | Domain |
|---|---|---|
| `EngineECU` | PartDef | hardware |
| `EngineControlSoftware` | PartDef (deploymentPackage) | software |
