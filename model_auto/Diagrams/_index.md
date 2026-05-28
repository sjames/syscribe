---
type: Package
name: Diagrams
---

Visual representations of the Engine ECU model for different stakeholder audiences.
Diagrams are stored as `type: Diagram` elements with a frontmatter manifest that links
SVG shapes and connector endpoints to their corresponding model elements by qualified name.

## Contents

| Diagram | Kind | Audience |
|---|---|---|
| `SystemOverview` | Mermaid block diagram | Systems architect, new team members |
| `SensorHierarchy` | SVG inheritance diagram | Software/hardware integration team |

## SystemOverview

Shows the high-level component topology: ECU hardware shell, software SWC cluster,
sensors (CPS, TPS, Lambda), actuators (throttle body, injectors), CAN transceiver,
hardware watchdog, and the powertrain CAN bus connection. The watchdog supervisory
line from the ECU to the watchdog IC is annotated with the 10 ms service window.

## SensorHierarchy

Depicts the abstract `Sensor` PartDef specialisation tree:

```
Sensor (abstract)
├── CrankshaftPositionSensor  — Hall-effect VR, teethCount
├── ThrottlePositionSensor    — dual-track potentiometric, supplyVoltage
└── LambdaSensor              — wideband UEGO, heatingPower
```

Used in design reviews to confirm that concrete sensor types correctly specialise the
abstract base and that common attributes (`operatingTempMin`, `operatingTempMax`,
`signalOutputType`) are inherited rather than duplicated.
